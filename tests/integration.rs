use bc_components::{PublicKeyBase, PrivateKeyBase};
use bc_envelope::prelude::*;
use bytes::Bytes;
use depo::{self, start_server, reset_db};
use depo_api::{StoreShareRequest, StoreShareResponse};
use reqwest::{self, StatusCode, Client};
use url::Url;
use std::time::Duration;
use tokio::time::sleep;
use hex_literal::hex;

// #[tokio::test]
// async fn test_scenario() {
//     // Reset the database
//     reset_db().await.unwrap();

//     // Start the server and wait for it to be ready
//     tokio::spawn(async {
//         start_server().await.unwrap();
//     });
//     sleep(Duration::from_secs(1)).await;

//     // Start the client
//     let client = Client::new();

//     let depo_public_key = get_public_key(&client).await.unwrap();

//     // Alice stores a share
//     let alice_private_key = PrivateKeyBase::new();
//     let alice_public_key = alice_private_key.public_keys();
//     let alice_data_1 = Bytes::from_static(&hex!("cafebabe"));
//     let request = StoreShareRequest::new(&alice_public_key, &alice_data_1);
//     let response_envelope = server_call(request, &alice_private_key, &depo_public_key, &client).await.unwrap();
//     let response = StoreShareResponse::try_from(response_envelope).unwrap();
//     let alice_receipt_1 = response.receipt();
//     println!("Alice receipt 1: {}", alice_receipt_1.ur_string());
// }

// fn url() -> Url {
//     let mut url = Url::parse("http://localhost").unwrap();
//     url.set_port(Some(5332)).unwrap();
//     url
// }

// async fn get_public_key(client: &Client) -> anyhow::Result<PublicKeyBase> {
//     let resp = client.get(url())
//         .send()
//         .await
//         .unwrap();

//     assert_eq!(resp.status(), StatusCode::OK);
//     let string = resp.text().await.unwrap();
//     let public_key = PublicKeyBase::from_ur_string(string)?;
//     Ok(public_key)
// }

// async fn server_call(request: impl EnvelopeEncodable, client_private_key: &PrivateKeyBase, depo_public_key: &PublicKeyBase, client: &Client) -> anyhow::Result<Envelope> {
//     let request = request.envelope();
//     let encrypted_request = request.sign_and_encrypt(client_private_key, depo_public_key)?;
//     let body = encrypted_request.ur_string();

//     let resp = client.get(url())
//         .body(body)
//         .send()
//         .await
//         .unwrap();

//     let encrypted_response = resp.text().await.unwrap();
//     let encrypted_response = Envelope::from_ur_string(encrypted_response)?;

//     let response = encrypted_response.verify_and_decrypt(depo_public_key, client_private_key)?;
//     assert_eq!(response.response_id().unwrap(), request.request_id().unwrap());
//     Ok(response)
// }
