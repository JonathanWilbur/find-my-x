use std::collections::HashMap;
use crate::storage::{Storage, SecretKey, Token};
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
    LocationSnapshot,
};

pub struct MemoryStorage {
    pub tokens_to_key: HashMap<Token, SecretKey>,
    pub locations: HashMap<SecretKey, Vec<SubmitLocationArg>>,
    pub intros: HashMap<SecretKey, IntroduceMyselfArg>,
    pub tokens_by_secret: HashMap<SecretKey, Vec<TokenInfo>>,
    pub tokens: HashMap<Vec<u8>, TokenInfo>,
}

impl MemoryStorage {

    pub fn new () -> Self {
        MemoryStorage{
            tokens_to_key: HashMap::new(),
            locations: HashMap::new(),
            intros: HashMap::new(),
            tokens_by_secret: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

}

#[tonic::async_trait]
impl Storage for MemoryStorage {

    async fn token_to_key (&self, token: &Token) -> anyhow::Result<Option<SecretKey>> {
        Ok(self.tokens_to_key.get(token).cloned())
    }

    async fn write_location (&mut self, arg: &SubmitLocationArg) -> anyhow::Result<()> {
        match self.locations.get_mut(arg.secret_key.as_slice()) {
            Some(locs) => {
                locs.push(arg.clone());
                Ok(())
            },
            None => {
                self.locations.insert(arg.secret_key.clone(), Vec::from([ arg.clone() ]));
                Ok(())
            },
        }
    }

    async fn write_intro (&mut self, arg: &IntroduceMyselfArg) -> anyhow::Result<()> {
        self.intros.insert(arg.secret_key.clone(), arg.clone());
        Ok(())
    }

    async fn write_token (&mut self, secret_key: &SecretKey, arg: &TokenInfo) -> anyhow::Result<()> {
        self.tokens.insert(arg.token.clone(), arg.clone());
        match self.tokens_by_secret.get_mut(secret_key.as_slice()) {
            Some(tokens) => {
                tokens.push(arg.clone());
                Ok(())
            },
            None => {
                self.tokens_by_secret.insert(secret_key.clone(), Vec::from([ arg.clone() ]));
                Ok(())
            },
        }
    }

    async fn revoke_token (&mut self, arg: &RevokeTokenArg) -> anyhow::Result<()> {
        self.tokens.remove(&arg.token.to_owned());
        self.tokens_by_secret.remove(&arg.secret_key.to_owned());
        Ok(())
    }

    async fn list_tokens (&self, arg: &ListTokensArg) -> anyhow::Result<ListTokensResult> {
        let maybe_secret_key: Option<SecretKey> = if arg.secret_key.len() > 0 {
            Some(arg.secret_key.clone())
        } else {
            self.token_to_key(&arg.token).await?
        };
        if maybe_secret_key.is_none() {
            return Ok(ListTokensResult {
                tokens: vec![],
            });
        }
        let secret_key = maybe_secret_key.unwrap();
        Ok(ListTokensResult {
            tokens: self.tokens_by_secret.get(secret_key.as_slice())
                .unwrap_or(&vec![])
                .to_owned(),
        })
    }

    async fn purge_location (&mut self, arg: &PurgeLocationArg) -> anyhow::Result<()> {
        let maybe_secret_key: Option<SecretKey> = self.token_to_key(&arg.token).await?;
        if let Some(secret_key) = maybe_secret_key {
            self.locations.remove(&secret_key);
        }
        Ok(())
    }

    async fn excommunicate (&mut self, arg: &RequestExcommunicationArg) -> anyhow::Result<()> {
        // FIXME: Implement a token -> secret key lookup
        todo!()
    }

    async fn list_locations (&self, arg: &ListLocationsArg) -> anyhow::Result<ListLocationsResult> {
        let maybe_secret_key: Option<SecretKey> = self.token_to_key(&arg.token).await?;
        if maybe_secret_key.is_none() {
            return Ok(ListLocationsResult {
                locations: vec![],
            });
        }
        let secret_key = maybe_secret_key.unwrap();
        Ok(ListLocationsResult {
            locations: self.locations.get(secret_key.as_slice())
                .unwrap_or(&vec![])
                .iter()
                .map(|loc| {
                    LocationSnapshot{
                        emergency: loc.emergency,
                        update_time: loc.update_time.to_owned(),
                        expected_next_update_time: loc.expected_next_update_time.to_owned(),
                        nearby_bluetooth_devices: loc.nearby_bluetooth_devices.to_owned(),
                        nearby_wifi_network: loc.nearby_wifi_network.to_owned(),
                        location: loc.location.to_owned(),
                        notes: loc.notes.to_owned(),
                        velocity: loc.velocity.to_owned(),
                    }
                })
                .collect()
        })

    }

    async fn get_storage_info (&self, arg: &GetStorageInfoArg) -> anyhow::Result<GetStorageInfoResult> {
        let maybe_secret_key: Option<SecretKey> = self.token_to_key(&arg.token).await?;
        if maybe_secret_key.is_none() {
            return Ok(GetStorageInfoResult {
                locations_count: 0,
                since: None,
                bytes_storage_consumed: 0,
                bytes_storage_limit: 0,
                locations_limit: 0,
            });
        }
        let secret_key = maybe_secret_key.unwrap();
        let empty = vec![];
        let locs = self.locations.get(&secret_key).unwrap_or(&empty);
        let since = locs
            .iter()
            .filter_map(|loc| loc.update_time.clone())
            .reduce(|acc, loc| {
                // We don't bother comparing nanoseconds. It's just not worth it.
                if loc.seconds < acc.seconds { loc } else { acc }
            });
        let locs_len = locs.len();
        Ok(GetStorageInfoResult {
            locations_count: locs_len as u32,
            since,
            bytes_storage_consumed: 0,
            bytes_storage_limit: 0,
            locations_limit: 100,
        })
    }

}