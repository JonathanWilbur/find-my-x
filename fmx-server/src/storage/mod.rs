pub mod memory;
use std::net::SocketAddr;

use crate::grpc::find_my_device::{
    IntroduceMyselfArg,
    RevokeTokenArg,
    RequestExcommunicationArg,
    ListLocationsResult,
    GetStorageInfoResult,
    NearbyWifiNetwork,
    NearbyBluetoothDevice,
    Location,
    Velocity,
    Permissions,
};
use chrono::prelude::*;

pub type Token = Vec<u8>;
pub type SecretKey = Vec<u8>;

#[derive(Debug, Clone)]
pub struct TokenEntry {
    pub secret_key: SecretKey,
    pub permissions: Permissions,
    pub not_before: DateTime<Utc>,
    pub not_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct LocationInsertion {
    pub update_time: DateTime<Utc>,
    pub expected_next_update_time: Option<DateTime<Utc>>,
    pub location: Option<Location>,
    pub velocity: Option<Velocity>,
    pub emergency: bool,
    pub notes: String,
    pub nearby_wifi_network: Vec<NearbyWifiNetwork>,
    pub nearby_bluetooth_devices: Vec<NearbyBluetoothDevice>,
    pub remote_addr: Option<SocketAddr>,
}

#[derive(Debug, Clone)]
pub struct IntroInsertion <'a> {
    pub secret_key: &'a SecretKey,
    pub token: &'a Token,
    pub remote_addr: Option<SocketAddr>,
    pub arg: &'a IntroduceMyselfArg,
}

pub struct LocationsFilter {
    pub limit: u32,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

#[tonic::async_trait]
pub trait Storage {

    async fn get_token_info (&self, arg: &Token) -> anyhow::Result<Option<TokenEntry>>;

    async fn write_location (&mut self, secret_key: &SecretKey, arg: &LocationInsertion) -> anyhow::Result<()>;

    async fn write_intro <'a> (&mut self, arg: &'a IntroInsertion) -> anyhow::Result<()>;

    async fn write_token (&mut self, secret_key: &SecretKey, arg: &TokenEntry) -> anyhow::Result<()>;

    async fn revoke_token (&mut self, arg: &RevokeTokenArg) -> anyhow::Result<()>;

    async fn list_tokens (&self, secret_key: &SecretKey) -> anyhow::Result<Vec<TokenEntry>>;

    async fn purge_location (&mut self, secret_key: &SecretKey) -> anyhow::Result<()>;

    async fn excommunicate (&mut self, arg: &RequestExcommunicationArg) -> anyhow::Result<()>;

    async fn list_locations (&self, secret_key: &SecretKey, filter: &LocationsFilter) -> anyhow::Result<ListLocationsResult>;

    async fn get_storage_info (&self, secret_key: &SecretKey) -> anyhow::Result<GetStorageInfoResult>;
}