mod config;
mod grpc;
mod logging;
mod storage;
use logging::get_default_log4rs_config;
use tonic::{transport::Server, Request, Response, Status};
use storage::{
    Storage,
    LocationInsertion, IntroInsertion,
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
use std::sync::Arc;
use tokio::sync::Mutex;
use log::warn;
use chrono::prelude::*;

fn grpc_timestamp_to_chrono (grpc_time: prost_types::Timestamp) -> Option<DateTime<Utc>> {
    match Utc.timestamp_opt(grpc_time.seconds, grpc_time.nanos as u32) {
        chrono::LocalResult::Single(dt) => Some(dt),
        _ => None,
    }
}

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
                .map(|t| grpc_timestamp_to_chrono(t).unwrap_or(Utc::now())),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_config(get_default_log4rs_config()).unwrap();
    let addr = "127.0.0.1:50051".parse()?;
    let device_service = DeviceServiceProvider {
        storage: Arc::new(Mutex::new(MemoryStorage::new())),
        config: Arc::new(Config{
            open_registration: true,
        }),
    };

    Server::builder()
        .add_service(DeviceServiceServer::new(device_service))
        .serve(addr)
        .await?;

    Ok(())
}