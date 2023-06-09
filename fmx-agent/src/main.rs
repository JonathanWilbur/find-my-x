use find_my_device::device_service_client::DeviceServiceClient;
use find_my_device::{
    IntroduceMyselfArg,
    SubmitLocationArg,
    Location,
};

pub mod find_my_device {
    tonic::include_proto!("findmydevice");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DeviceServiceClient::connect("http://127.0.0.1:50051").await?;

    let token: Vec<u8> = {
        let request = tonic::Request::new(IntroduceMyselfArg {
            ..Default::default()
        });
        let response = client.introduce_myself(request).await?;
        let resp = response.into_inner();
        if resp.nice_to_meet_you {
            println!("Server said hello.");
        }
        resp.your_token
    };

    {
        let request = tonic::Request::new(SubmitLocationArg {
            token,
            emergency: true,
            location: Some(Location{
                degrees_latitude: 30.0832,
                degress_longitude: -81.4028,
                meters_elevation: 0.0,
            }),
            ..Default::default()
        });
    
        let response = client.submit_location(request).await?;
        println!("RESPONSE={:?}", response);
    }

    Ok(())
}