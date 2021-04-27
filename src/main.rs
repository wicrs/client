use wicrs_server::signing;

use pgp::crypto::HashAlgorithm;
use pgp::packet::LiteralData;
use pgp::types::KeyTrait;
use pgp::Deserializable;
use pgp::Message as OpenPGPMessage;
use pgp::SignedPublicKey;

use futures_util::{SinkExt, StreamExt};

use tokio_tungstenite::tungstenite::Message;

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
    let request = tokio_tungstenite::tungstenite::http::request::Builder::new()
        .header("pgp-fingerprint", fingerprint)
        .uri("ws://localhost:8080/v2/websocket")
        .body(())
        .unwrap();
    let mut ws_stream = tokio_tungstenite::connect_async(request).await.unwrap().0;
    if let Some(Ok(msg)) = ws_stream.next().await {
        let key_message = OpenPGPMessage::from_string(&msg.to_text().unwrap())
            .unwrap()
            .0;
        key_message.verify(&server_key).unwrap();
        let key = key_message.get_literal().unwrap().to_string().unwrap();
        let key_msg = OpenPGPMessage::Literal(LiteralData::from_str("websocket_connect", &key))
            .sign(&key_pair.secret_key, String::new, HashAlgorithm::SHA2_256)
            .unwrap();
        let key_str = key_msg
            .to_armored_string(None)
            .unwrap()
            .replace('\n', "\r\n");
        ws_stream.send(Message::text(key_str)).await.unwrap();
    }
    ws_stream.close(None).await.unwrap();
}
