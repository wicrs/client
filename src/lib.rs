use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use wicrs_api::{config::ClientConfig, wicrs_server::{ID, channel::Message, hub::Hub, user::User}};
pub use wicrs_api::{self, Result};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Cache {
    pub user: User,
    pub hubs: HashMap<ID, Hub>,
    pub other_users: HashMap<ID, User>,
    pub messages: HashMap<ID, HashMap<ID, Message>>,
    pub client_config: ClientConfig,
}

pub struct Client {
    pub http_client: wicrs_api::Client,
    pub current_user: User,
    pub known_hubs: HashMap<ID, Hub>,
    pub known_users: HashMap<ID, User>,
    pub messages: HashMap<ID, HashMap<ID, Message>>,
    pub client_config: ClientConfig,
}

impl Client {
    pub async fn from_client_config(config: ClientConfig) -> Result<Self> {
        let http_client = wicrs_api::Client::from_config(config.clone())?;
        let user = http_client.get_user().await?;
        let mut known_hubs = HashMap::new();
        for hub_id in user.in_hubs.clone() {
            if let Ok(hub) = http_client.get_hub(&hub_id).await {
                known_hubs.insert(hub_id, hub);
            }
        }
        Ok(Self {
            current_user: user,
            http_client,
            known_hubs,
            known_users: HashMap::new(),
            client_config: config,
            messages: HashMap::new(),
        })
    }

    pub fn from_cache(cache: Cache) -> Result<Self> {
        Ok(Self {
            http_client: wicrs_api::Client::from_config(cache.client_config.clone())?,
            current_user: cache.user,
            known_hubs: cache.hubs,
            known_users: cache.other_users,
            messages: cache.messages,
            client_config: cache.client_config,

        })
    }
}
