mod config;
mod grpc;
mod logging;
mod storage;
mod utils;
mod web;
use logging::get_default_log4rs_config;
use tonic::{transport::Server, Request, Response, Status};
use storage::{
    Storage,
    LocationInsertion,
    IntroInsertion,
    LocationsFilter,
    Token,
};
use config::Config;
use storage::memory::MemoryStorage;
use grpc::find_my_device::device_service_server::{DeviceService, DeviceServiceServer};
use grpc::find_my_device::{
    SubmitLocationArg,
    SubmitLocationResult,
    // StreamServerEventsArg,
    // ServerEvent,
    IntroduceMyselfArg,
    IntroduceMyselfResult,
};
use warp::Filter;
use warp::http::StatusCode;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::warn;
use chrono::prelude::*;
use web::{LocationsPage, Props};
use std::convert::Infallible;
use std::rc::Rc;
use utils::grpc_timestamp_to_chrono;

#[derive(Clone)]
pub struct DeviceServiceProvider <S: Storage> {
    pub storage: Arc<Mutex<S>>,
    pub config: Arc<Config>,
}

#[tonic::async_trait]
impl <S: Storage + Send + Sync + 'static> DeviceService for DeviceServiceProvider <S> {

    async fn submit_location (
        &self,
        request: Request<SubmitLocationArg>,
    ) -> Result<Response<SubmitLocationResult>, Status> {
        let maybe_remote_addr = request.remote_addr();
        let req = request.into_inner();
        let mut storage = self.storage.lock().await;
        let maybe_token_info = storage.get_token_info(&req.token).await
            .map_err(|_| Status::internal("Database failure."))?;
        if maybe_token_info.is_none() {
            return Err(Status::unauthenticated("Unauthenticated"));
        }
        let token_info = maybe_token_info.unwrap();
        if req.emergency {
            warn!("Emergency announced by {:?}", token_info.secret_key);
        }
        let insertion = LocationInsertion{
            emergency: req.emergency,
            update_time: Utc::now(),
            expected_next_update_time: req.expected_next_update_time
                .map(|t| grpc_timestamp_to_chrono(&t).unwrap_or(Utc::now())),
            location: req.location,
            notes: req.notes,
            velocity: req.velocity,
            nearby_bluetooth_devices: req.nearby_bluetooth_devices,
            nearby_wifi_network: req.nearby_wifi_network,
            remote_addr: maybe_remote_addr,
        };
        match storage.write_location(&token_info.secret_key, &insertion).await {
            Ok(_) => Ok(Response::new(SubmitLocationResult {
                recorded: true,
                excommunicated: false,
                remote_wipe: false,
            })),
            Err(_) => Err(Status::internal("Database failure.")),
        }
    }

    async fn introduce_myself (
        &self,
        request: Request<IntroduceMyselfArg>,
    ) -> Result<Response<IntroduceMyselfResult>, Status> {
        let maybe_remote_addr = request.remote_addr();
        let req = request.into_inner();
        if !self.config.open_registration {
            // TODO: Verify registration key.
            // return Err(Status::invalid_argument("Secret key must be 16 bytes"));
        }
        let random_bytes = rand::random::<[u8; 32]>();
        let secret_key = Vec::from(&random_bytes[0..16]);
        let token = Vec::from(&random_bytes[16..]);
        let insertion = IntroInsertion{
            remote_addr: maybe_remote_addr,
            secret_key: &secret_key,
            token: &token,
            arg: &req,
        };
        let mut storage = self.storage.lock().await;
        match storage.write_intro(&insertion).await {
            Ok(_) => Ok(Response::new(IntroduceMyselfResult {
                nice_to_meet_you: true,
                your_token: token,
            })),
            Err(_) => Err(Status::internal("Database failure.")),
        }
    }

    // findmydevice.rs(955, 9): `StreamServerEventsStream` from trait
    // findmydevice.rs(960, 9): `stream_server_events` from trait

}

// fn render_locations_path (token: String) -> String {

// }

async fn render_locations_path <S: Storage> (
    token_str: String,
    storage: Arc<Mutex<S>>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    // tokio::time::sleep(Duration::from_secs(seconds)).await;
    // Ok(format!("I waited {} seconds!", seconds))
    let token: Token = match hex::decode(token_str) {
        Ok(h) => h,
        Err(_) => return Ok(Box::new(warp::reply::with_status(String::from("Malformed token"), StatusCode::BAD_REQUEST))),
    };
    let store = storage.lock().await;
    let maybe_token_info = match store.get_token_info(&token).await {
        Ok(t) => t,
        Err(e) => return Ok(Box::new(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
    };
    if maybe_token_info.is_none() {
        return Ok(Box::new(warp::reply::with_status(String::from("Unauthorized"), StatusCode::UNAUTHORIZED)));
    }
    let token_info = maybe_token_info.unwrap();
    if !token_info.permissions.read_locations {
        return Ok(Box::new(warp::reply::with_status(String::from("Forbidden"), StatusCode::FORBIDDEN)));
    }
    let filter = LocationsFilter {
        limit: 100,
        since: None,
        until: None,
    };
    let locs = match store.list_locations(&token_info.secret_key, &filter).await {
        Ok(l) => l,
        Err(e) => return Ok(Box::new(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
    };
    let renderer = yew::ServerRenderer::<LocationsPage>::with_props(move || Props {
        locations: locs.locations.into_iter().map(|l| Rc::new(l)).collect(),
    });
    // .hydratable(false) gets rid of the HTML comments.
    let rendered = renderer.hydratable(false).render().await;
    Ok(Box::new(warp::reply::html(rendered)))
}

fn with_storage <S: Storage + Sync + Send> (
    storage: Arc<Mutex<S>>,
) -> impl Filter<Extract = (Arc<Mutex<S>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_config(get_default_log4rs_config()).unwrap();
    let addr = "127.0.0.1:50051".parse()?;
    let storage = Arc::new(Mutex::new(MemoryStorage::new()));
    let device_service = DeviceServiceProvider {
        storage: storage.clone(),
        config: Arc::new(Config{
            open_registration: true,
        }),
    };

    tokio::spawn(Server::builder()
        .add_service(DeviceServiceServer::new(device_service))
        .serve(addr));

    let locations_path = warp::path!("locations" / String)
        .and(with_storage(storage))
        .and_then(|token, storage| {
            render_locations_path(token, storage)
        });

    warp::serve(locations_path)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}