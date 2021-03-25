use std::collections::HashMap;

use wicrs_api::wicrs_server::{ID, hub::Hub, user::User};

pub struct Client {
    http_client: wicrs_api::Client,
    current_user: User,
    known_servers: HashMap<ID, Hub>,
}
