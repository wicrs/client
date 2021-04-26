use wicrs_server::signing;

use pgp::crypto::HashAlgorithm;
use pgp::packet::LiteralData;
use pgp::types::KeyTrait;
use pgp::Deserializable;
use pgp::Message as OpenPGPMessage;
use pgp::SignedPublicKey;

#[tokio::main]
async fn main() {
    let key_pair = signing::KeyPair::load_or_create(
        "WICRS Client <client@wic.rs>",
        signing::SECRET_KEY_PATH,
        signing::PUBLIC_KEY_PATH,
    )
    .await
    .unwrap();
    let read = std::fs::read_to_string("data/server_public_key.asc").unwrap();
    let server_key = SignedPublicKey::from_string(&read).unwrap().0;
    let fingerprint = hex::encode(key_pair.secret_key.fingerprint());
    let string = format!("websocket_connect {}", fingerprint);
    let message = OpenPGPMessage::Literal(LiteralData::from_str("websocket_connect", &string))
        .sign(&key_pair.secret_key, String::new, HashAlgorithm::SHA2_256)
        .unwrap();
    let response = reqwest::Client::default()
        .request(
            reqwest::Method::GET,
            "http://localhost:8080/v2/websocket_init",
        )
        .header("pgp-fingerprint", &fingerprint)
        .body(message.to_armored_string(None).unwrap())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let message = OpenPGPMessage::from_string(&response).unwrap().0;
    message.verify(&server_key).unwrap();
    let key = message.get_literal().unwrap().to_string().unwrap();
    let request = tokio_tungstenite::tungstenite::http::request::Builder::new()
        .header("pgp-fingerprint", fingerprint)
        .uri("http://localhost:8080/v2/websocket")
        .header("signed-ws-key", key)
        .body(())
        .unwrap();
    let mut ws_stream = tokio_tungstenite::connect_async(request).await.unwrap().0;
    ws_stream.close(None).await.unwrap();
}
