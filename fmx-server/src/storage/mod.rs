mod memory;
use crate::grpc::find_my_device::{
    SubmitLocationArg,
    IntroduceMyselfArg,
    TokenInfo,
    RevokeTokenArg,
    ListTokensArg,
    ListTokensResult,
    PurgeLocationArg,
    RequestExcommunicationArg,
    ListLocationsArg,
    ListLocationsResult,
    GetStorageInfoArg,
    GetStorageInfoResult,
};

pub type Token = Vec<u8>;
pub type SecretKey = Vec<u8>;

#[tonic::async_trait]
pub trait Storage {

    async fn token_to_key (&self, arg: &Token) -> anyhow::Result<Option<SecretKey>>;

    async fn write_location (&mut self, arg: &SubmitLocationArg) -> anyhow::Result<()>;

    async fn write_intro (&mut self, arg: &IntroduceMyselfArg) -> anyhow::Result<()>;

    async fn write_token (&mut self, secret_key: &SecretKey, arg: &TokenInfo) -> anyhow::Result<()>;

    async fn revoke_token (&mut self, arg: &RevokeTokenArg) -> anyhow::Result<()>;

    async fn list_tokens (&self, arg: &ListTokensArg) -> anyhow::Result<ListTokensResult>;

    async fn purge_location (&mut self, arg: &PurgeLocationArg) -> anyhow::Result<()>;

    async fn excommunicate (&mut self, arg: &RequestExcommunicationArg) -> anyhow::Result<()>;

    async fn list_locations (&self, arg: &ListLocationsArg) -> anyhow::Result<ListLocationsResult>;

    async fn get_storage_info (&self, arg: &GetStorageInfoArg) -> anyhow::Result<GetStorageInfoResult>;
}