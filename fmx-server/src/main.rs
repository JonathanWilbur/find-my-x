mod grpc;
mod storage;
use tonic::{transport::Server, Request, Response, Status};

use grpc::find_my_device::device_service_server::{DeviceService, DeviceServiceServer};
// use findmydevice::DeviceService::{DeviceService, DeviceServiceServer};
use grpc::find_my_device::{
    SubmitLocationArg,
    SubmitLocationResult,
    StreamServerEventsArg,
    ServerEvent,
    IntroduceMyselfArg,
    IntroduceMyselfResult,
    GetMyIdArg,
    GetMyIdResult,
};

// rpc SubmitLocation (SubmitLocationArg) returns SubmitLocationResult;
// rpc StreamServerEvents (StreamServerEventsArg) returns (stream ServerEvent);
// rpc IntroduceMyself (IntroduceMyselfArg) returns IntroduceMyselfResult;

// // This operation basically just exists so low-power / IoT devices with no
// // support for cryptographic operations can just request their deviceId from
// // the server.
// rpc GetMyId (GetMyIdArg) returns GetMyIdResult;

#[derive(Debug, Default)]
pub struct MockDeviceService;

#[tonic::async_trait]
impl DeviceService for MockDeviceService {

    async fn submit_location (
        &self,
        request: Request<SubmitLocationArg>,
    ) -> Result<Response<SubmitLocationResult>, Status> {
        let req = request.into_inner();
        if req.emergency {
            println!("Not an emergency.");
        } else {
            println!("Not an emergency.");
        }
        Err(Status::new(tonic::Code::NotFound, "Unimplemented"))
    }

    async fn introduce_myself (
        &self,
        request: Request<IntroduceMyselfArg>,
    ) -> Result<Response<IntroduceMyselfResult>, Status> {
        Err(Status::new(tonic::Code::NotFound, "Unimplemented"))
    }

    async fn get_my_id (
        &self,
        request: Request<GetMyIdArg>,
    ) -> Result<Response<GetMyIdResult>, Status> {
        Err(Status::new(tonic::Code::NotFound, "Unimplemented"))
    }

    // findmydevice.rs(955, 9): `StreamServerEventsStream` from trait
    // findmydevice.rs(960, 9): `stream_server_events` from trait

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let device_service = MockDeviceService::default();

    Server::builder()
        .add_service(DeviceServiceServer::new(device_service))
        .serve(addr)
        .await?;

    Ok(())
}