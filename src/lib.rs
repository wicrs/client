use std::collections::HashMap;

use wicrs_api::{
    wicrs_server::{hub::Hub, user::User, ID},
    ClientBuilder, Result,
};

pub struct Client {
    pub http_client: wicrs_api::Client,
    pub current_user: User,
    pub known_hubs: HashMap<ID, Hub>,
    pub known_users: HashMap<ID, User>,
}

impl Client {
    pub async fn from_builder(builder: ClientBuilder) -> Result<Self> {
        let http_client = builder.build().await?;
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
        })
    }
}
