use tonic::{transport::Server, Request, Response, Status};

use find_my_device::device_service_client::DeviceServiceClient;
// use findmydevice::DeviceService::{DeviceService, DeviceServiceServer};
use find_my_device::{
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

pub mod find_my_device {
    tonic::include_proto!("findmydevice");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DeviceServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(SubmitLocationArg {
        emergency: true,
        ..Default::default()
    });

    let response = client.submit_location(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}