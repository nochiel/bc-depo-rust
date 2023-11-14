use bc_components::{PrivateKeyBase, ARID, PublicKeyBase};
use bytes::Bytes;
use depo_api::{request::store_share::StoreShareRequest, GetSharesRequest, StoreShareResponse, GetSharesResponse, DeleteSharesRequest, UpdateRecoveryRequest, GetRecoveryRequest, GetRecoveryResponse, UpdateKeyRequest, StartRecoveryRequest, StartRecoveryResponse, FinishRecoveryRequest, DeleteAccountRequest};
use indoc::indoc;
use bc_envelope::prelude::*;
use hex_literal::hex;
use depo::{Depo, reset_db};

#[tokio::test]
async fn test_mem_depo() {
    let depo = Depo::new_in_memory();
    test_depo(depo).await;
}

#[tokio::test]
async fn test_db_depo() {
    reset_db().await.unwrap();
    let depo = Depo::new_db().await.unwrap();
    test_depo(depo).await;
}

#[test]
fn test_store_share_request() {
    let id = ARID::from_data_ref(hex_literal::hex!("8712dfac3d0ebfa910736b2a9ee39d4b68f64222a77bcc0074f3f5f1c9216d30")).unwrap();
    let private_key = PrivateKeyBase::new();
    let key = private_key.public_keys();
    let data = Bytes::from_static(b"data");
    let request = StoreShareRequest::new_opt(id, key, data);
    assert_eq!(request.clone().envelope().format(),
        indoc! {r#"
        request(ARID(8712dfac)) [
            'body': «"storeShare"» [
                ❰"data"❱: Bytes(4)
                ❰"key"❱: PublicKeyBase
            ]
        ]
        "#}.trim()
    );

    let server_private_key = PrivateKeyBase::new();
    let server_public_key = server_private_key.public_keys();

    let encrypted_request = request.clone().envelope().sign_and_encrypt(&private_key, &server_public_key).unwrap();
    assert_eq!(encrypted_request.format(),
        indoc! {r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
        ]
        "#}.trim()
    );

    let signed_request = encrypted_request
        .decrypt_to_recipient(&server_private_key).unwrap()
        .unwrap_envelope().unwrap();
    assert_eq!(signed_request.format(),
        indoc! {r#"
        {
            request(ARID(8712dfac)) [
                'body': «"storeShare"» [
                    ❰"data"❱: Bytes(4)
                    ❰"key"❱: PublicKeyBase
                ]
            ]
        } [
            'verifiedBy': Signature
        ]
        "#}.trim()
    );
}

async fn server_call(request: impl EnvelopeEncodable, client_private_key: &PrivateKeyBase, depo_public_key: &PublicKeyBase, depo: &Depo) -> anyhow::Result<Envelope> {
    let request = request.envelope();
    let encrypted_request = request.sign_and_encrypt(client_private_key, depo_public_key)?;
    let encrypted_response = depo.handle_request(encrypted_request).await?;
    let response = encrypted_response.verify_and_decrypt(depo_public_key, client_private_key)?;
    assert_eq!(response.response_id().unwrap(), request.request_id().unwrap());
    Ok(response)
}

async fn test_depo(depo: Depo) {
    let depo_public_key = depo.public_key();

    // Alice stores a share
    let alice_private_key = PrivateKeyBase::new();
    let alice_public_key = alice_private_key.public_keys();
    let alice_data_1 = Bytes::from_static(&hex!("cafebabe"));
    let request = StoreShareRequest::new(&alice_public_key, &alice_data_1);
    let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let alice_receipt_1 = response.receipt();

    // Bob stores a share
    let bob_private_key = PrivateKeyBase::new();
    let bob_public_key = bob_private_key.public_keys();
    let bob_data_1 = Bytes::from_static(&hex!("deadbeef"));
    let request = StoreShareRequest::new(&bob_public_key, &bob_data_1);
    let response_envelope = server_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let bob_receipt_1 = response.receipt();

    // Alice retrieves her share
    let request = GetSharesRequest::new(&alice_public_key, vec![&alice_receipt_1]);
    let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    let alice_retrieved_data_1 = response.data_for_receipt(&alice_receipt_1).unwrap();
    assert_eq!(alice_retrieved_data_1, alice_data_1);

    // Bob retrieves his share
    let request = GetSharesRequest::new(&bob_public_key, vec![&bob_receipt_1]);
    let response_envelope = server_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    let bob_retrieved_data_1 = response.data_for_receipt(&bob_receipt_1).unwrap();
    assert_eq!(bob_retrieved_data_1, bob_data_1);

    // Alice stores a second share
    let alice_data_2 = Bytes::from_static(&hex!("cafef00d"));
    let request = StoreShareRequest::new(&alice_public_key, &alice_data_2);
    let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let alice_receipt_2 = response.receipt();

    // Alice retrieves her second share
    let request = GetSharesRequest::new(&alice_public_key, vec![&alice_receipt_2]);
    let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    let alice_retrieved_data_2 = response.data_for_receipt(&alice_receipt_2).unwrap();
    assert_eq!(alice_retrieved_data_2, alice_data_2);

    // Alice retrieves both her shares identified only by her public key
    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    assert_eq!(response.receipt_to_data().len(), 2);

    // Bob attempts to retrieve one of Alice's shares
    let request = GetSharesRequest::new(&bob_public_key, vec![&alice_receipt_1]);
    let response_envelope = server_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    assert_eq!(response.receipt_to_data().len(), 0);

    // Someone attempts to retrieve all shares from a nonexistent account
    let nonexistent_private_key = PrivateKeyBase::new();
    let nonexistent_public_key = nonexistent_private_key.public_keys();
    let request = GetSharesRequest::new(&nonexistent_public_key, vec![]);
    let response = server_call(request, &nonexistent_private_key, depo_public_key, &depo).await;
    assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // Someone attempts to retrieve all shares from Alice's account using her public key
    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response = server_call(request, &nonexistent_private_key, depo_public_key, &depo).await;
    assert_eq!(response.err().unwrap().to_string(), "could not verify a signature");

    // Alice attempts to retrieve her shares using the incorrect depo public key
    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response = server_call(request, &alice_private_key, &nonexistent_public_key, &depo).await;
    assert_eq!(response.err().unwrap().to_string(), "no recipient matches the given key");

    // Alice stores a share she's previously stored (idempotent)
    let request = StoreShareRequest::new(&alice_public_key, alice_data_1);
    let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let alice_receipt_3 = response.receipt();
    assert_eq!(alice_receipt_3, alice_receipt_1);

    // Alice deletes one of her shares
    let request = DeleteSharesRequest::new(&alice_public_key, vec![&alice_receipt_1]);
    server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();

    // let request = GetSharesRequest::new(&alice_public_key, vec![]);
    // let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    // let response = GetSharesResponse::try_from(response_envelope).unwrap();
    // assert_eq!(response.receipt_to_data().len(), 1);
    // let alice_retrieved_data_2 = response.data_for_receipt(&alice_receipt_2).unwrap();
    // assert_eq!(alice_retrieved_data_2, alice_data_2);

    // // Alice attempts to delete a share she already deleted (idempotent)
    // let request = DeleteSharesRequest::new(&alice_public_key, vec![&alice_receipt_1]);
    // server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();

    // let request = GetSharesRequest::new(&alice_public_key, vec![]);
    // let response_envelope = server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    // let response = GetSharesResponse::try_from(response_envelope).unwrap();
    // assert_eq!(response.receipt_to_data().len(), 1);
    // let alice_retrieved_data_2 = response.data_for_receipt(&alice_receipt_2).unwrap();
    // assert_eq!(alice_retrieved_data_2, alice_data_2);

    // // Bob adds a recovery method
    // let bob_recovery = "bob@example.com";
    // let request = UpdateRecoveryRequest::new(&bob_public_key, Some(bob_recovery));
    // server_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();

    // let request = GetRecoveryRequest::new(&bob_public_key);
    // let response_envelope = server_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    // let response = GetRecoveryResponse::try_from(response_envelope).unwrap();
    // assert_eq!(response.recovery(), Some(bob_recovery));

    // // Alice attempts to add a non-unique recovery method
    // let request = UpdateRecoveryRequest::new(&alice_public_key, Some(bob_recovery));
    // let response = server_call(request, &alice_private_key, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "recovery method already exists");

    // // Someone attempts to retrieve the recovery method for a nonexistent account
    // let request = GetRecoveryRequest::new(&nonexistent_public_key);
    // let response = server_call(request, &nonexistent_private_key, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // // Alice updates her public key to a new one
    // let alice_private_key_2 = PrivateKeyBase::new();
    // let alice_public_key_2 = alice_private_key_2.public_keys();
    // let request = UpdateKeyRequest::new(&alice_public_key, &alice_public_key_2);
    // server_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();

    // // Alice can no longer retrieve her shares using the old public key
    // let request = GetSharesRequest::new(&alice_public_key, vec![]);
    // let response = server_call(request, &alice_private_key, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // // Alice must now use her new public key
    // let request = GetSharesRequest::new(&alice_public_key_2, vec![]);
    // let response_envelope = server_call(request, &alice_private_key_2, depo_public_key, &depo).await.unwrap();
    // let response = GetSharesResponse::try_from(response_envelope).unwrap();
    // assert_eq!(response.receipt_to_data().len(), 1);

    // // Bob has lost his public key, so he wants to replace it with a new one
    // let bob_private_key_2 = PrivateKeyBase::new();
    // let bob_public_key_2 = bob_private_key_2.public_keys();

    // // Bob requests transfer using an incorrect recovery method
    // let incorrect_recovery = "wrong@example.com";
    // let request = StartRecoveryRequest::new(&bob_public_key_2, incorrect_recovery);
    // let response = server_call(request, &bob_private_key_2, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "unknown recovery");

    // // Bob requests a transfer using the correct recovery method
    // let request = StartRecoveryRequest::new(&bob_public_key_2, bob_recovery);
    // let response_envelope = server_call(request, &bob_private_key_2, depo_public_key, &depo).await.unwrap();
    // let response = StartRecoveryResponse::try_from(response_envelope).unwrap();

    // // The recovery continuation is both signed by the server and encrypted to the server, and is also time-limited.
    // // It is sent to Bob's recovery contact method, which acts as a second factor.
    // // Once in possession of the recovery continuation, Bob can use it to finish the recovery process.
    // let continuation = response.continuation();

    // // Bob uses the recovery continuation to finish setting his new public key
    // let request = FinishRecoveryRequest::new(&bob_public_key_2, continuation);
    // server_call(request, &bob_private_key_2, depo_public_key, &depo).await.unwrap();

    // // Bob can no longer retrieve his shares using the old public key
    // let request = GetSharesRequest::new(&bob_public_key, vec![]);
    // let response = server_call(request, &bob_private_key, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // // Bob must now use his new public key
    // let request = GetSharesRequest::new(&bob_public_key_2, vec![]);
    // let response_envelope = server_call(request, &bob_private_key_2, depo_public_key, &depo).await.unwrap();
    // let response = GetSharesResponse::try_from(response_envelope).unwrap();
    // assert_eq!(response.receipt_to_data().len(), 1);

    // // Bob decides to delete his account
    // let request = DeleteAccountRequest::new(&bob_public_key_2);
    // server_call(request, &bob_private_key_2, depo_public_key, &depo).await.unwrap();

    // // Bob can no longer retrieve his shares using the new public key
    // let request = GetSharesRequest::new(&bob_public_key_2, vec![]);
    // let response = server_call(request, &bob_private_key_2, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // // Attempting to retrieve his recovery method now throws an error
    // let request = GetRecoveryRequest::new(&bob_public_key_2);
    // let response = server_call(request, &bob_private_key_2, depo_public_key, &depo).await;
    // assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // // Deleting an account is idempotent
    // let request = DeleteAccountRequest::new(&bob_public_key_2);
    // server_call(request, &bob_private_key_2, depo_public_key, &depo).await.unwrap();
}
