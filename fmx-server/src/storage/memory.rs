use std::collections::HashMap;
use std::net::SocketAddr;
use crate::storage::{
    Storage,
    SecretKey,
    Token,
    LocationInsertion,
    IntroInsertion,
    TokenEntry,
    LocationsFilter,
};
use crate::grpc::find_my_device::{
    RevokeTokenArg,
    ListLocationsResult,
    GetStorageInfoResult,
    LocationSnapshot,
    Permissions,
};
use chrono::prelude::*;

#[derive(Clone)]
pub struct Introduction {
    pub remote_addr: Option<SocketAddr>,
    pub registration_key: Vec<u8>,
    pub remote_wipe_enabled: bool,
    pub can_read_nearby_devices: bool,
}

#[derive(Clone)]
pub struct MemoryStorage {
    pub locations: HashMap<SecretKey, Vec<LocationInsertion>>,
    pub intros: HashMap<SecretKey, Introduction>,
    pub tokens_by_secret: HashMap<SecretKey, Vec<Token>>,
    pub tokens: HashMap<Token, TokenEntry>,
}

impl MemoryStorage {

    pub fn new () -> Self {
        MemoryStorage{
            locations: HashMap::new(),
            intros: HashMap::new(),
            tokens_by_secret: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

}

#[tonic::async_trait]
impl Storage for MemoryStorage {

    async fn get_token_info (&self, token: &Token) -> anyhow::Result<Option<TokenEntry>> {
        Ok(self.tokens.get(token).cloned())
    }

    async fn write_location (&mut self, secret_key: &SecretKey, arg: &LocationInsertion) -> anyhow::Result<()> {
        match self.locations.get_mut(secret_key.as_slice()) {
            Some(locs) => {
                locs.push(arg.clone());
                Ok(())
            },
            None => {
                self.locations.insert(secret_key.clone(), Vec::from([ arg.clone() ]));
                Ok(())
            },
        }
    }

    async fn write_intro <'a> (&mut self, arg: &'a IntroInsertion) -> anyhow::Result<()> {
        self.intros.insert(arg.secret_key.clone(), Introduction{
            remote_addr: arg.remote_addr.clone(),
            registration_key: arg.arg.registration_key.clone(),
            remote_wipe_enabled: arg.arg.remote_wipe_enabled,
            can_read_nearby_devices: arg.arg.can_read_nearby_devices,
        });
        self.tokens.insert(arg.token.clone(), TokenEntry {
            secret_key: arg.secret_key.clone(),
            permissions: Permissions{
                list_tokens: false,
                write_locations: true,
                read_locations: true, // TODO: Set this to false after testing is done.
                nearby: true,
                stats: false,
                wipe: false,
            },
            not_before: Utc::now(),
            not_after: None,
        });
        match self.tokens_by_secret.get_mut(arg.secret_key.as_slice()) {
            Some(tokens) => {
                tokens.push(arg.token.clone());
            },
            None => {
                self.tokens_by_secret.insert(arg.secret_key.clone(), Vec::from([ arg.token.clone() ]));
            },
        }
        Ok(())
    }

    async fn write_token (&mut self, token: &Token, arg: &TokenEntry) -> anyhow::Result<()> {
        self.tokens.insert(token.clone(), arg.clone());
        match self.tokens_by_secret.get_mut(arg.secret_key.as_slice()) {
            Some(tokens) => {
                tokens.push(token.clone());
                Ok(())
            },
            None => {
                self.tokens_by_secret.insert(arg.secret_key.clone(), Vec::from([ token.clone() ]));
                Ok(())
            },
        }
    }

    async fn revoke_token (&mut self, arg: &RevokeTokenArg) -> anyhow::Result<()> {
        self.tokens.remove(&arg.token.to_owned());
        self.tokens_by_secret.remove(&arg.secret_key.to_owned());
        Ok(())
    }

    async fn list_tokens (&self, secret_key: &SecretKey) -> anyhow::Result<Vec<TokenEntry>> {
        let tokens = self.tokens_by_secret.get(secret_key.as_slice())
            .cloned()
            .unwrap_or(Vec::new());
        let token_infos = tokens
            .iter()
            .filter_map(|t| self.tokens.get(t).cloned())
            .collect();
        Ok(token_infos)
    }

    async fn purge_location (&mut self, secret_key: &SecretKey) -> anyhow::Result<()> {
        self.locations.remove(secret_key);
        Ok(())
    }

    async fn list_locations (&self, secret_key: &SecretKey, filter: &LocationsFilter) -> anyhow::Result<ListLocationsResult> {
        Ok(ListLocationsResult {
            locations: self.locations.get(secret_key.as_slice())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|loc| {
                    if let Some(since) = filter.since {
                        if loc.update_time < since {
                            return None;
                        }
                    }
                    if let Some(until) = filter.until {
                        if loc.update_time > until {
                            return None;
                        }
                    }
                    Some(LocationSnapshot{
                        emergency: loc.emergency,
                        // TODO: Refactor into a function
                        update_time: Some(prost_types::Timestamp{
                            seconds: loc.update_time.timestamp(),
                            nanos: loc.update_time.nanosecond() as i32,
                        }),
                        expected_next_update_time: loc.expected_next_update_time.map(|t| prost_types::Timestamp {
                            seconds: t.timestamp(),
                            nanos: t.nanosecond() as i32,
                        }),
                        nearby_bluetooth_devices: loc.nearby_bluetooth_devices.to_owned(),
                        nearby_wifi_network: loc.nearby_wifi_network.to_owned(),
                        location: loc.location.to_owned(),
                        notes: loc.notes.to_owned(),
                        velocity: loc.velocity.to_owned(),
                    })
                })
                .take(filter.limit as usize)
                .collect()
        })

    }

    async fn get_storage_info (&self, secret_key: &SecretKey) -> anyhow::Result<GetStorageInfoResult> {
        let empty = vec![];
        let locs = self.locations.get(secret_key).unwrap_or(&empty);
        let since = locs
            .iter()
            .filter_map(|loc| Some(loc.update_time))
            .reduce(|acc, loc| {
                // We don't bother comparing nanoseconds. It's just not worth it.
                if loc.timestamp() < acc.timestamp() { loc } else { acc }
            });
        let locs_len = locs.len();
        Ok(GetStorageInfoResult {
            locations_count: locs_len as u32,
            since: since.map(|t| prost_types::Timestamp {
                seconds: t.timestamp(),
                nanos: t.nanosecond() as i32,
            }),
            bytes_storage_consumed: 0,
            bytes_storage_limit: 0,
            locations_limit: 100,
        })
    }

}