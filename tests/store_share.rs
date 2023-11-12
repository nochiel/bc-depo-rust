use bc_components::{PrivateKeyBase, ARID, PublicKeyBase};
use bytes::Bytes;
use depo_api::{request::store_share::StoreShareRequest, GetSharesRequest, StoreShareResponse, GetSharesResponse, DeleteSharesRequest};
use indoc::indoc;
use bc_envelope::prelude::*;
use hex_literal::hex;
use depo::Depo;

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

async fn client_call(request: impl EnvelopeEncodable, client_private_key: &PrivateKeyBase, depo_public_key: &PublicKeyBase, depo: &Depo) -> anyhow::Result<Envelope> {
    let request = request.envelope();
    let encrypted_request = request.sign_and_encrypt(client_private_key, depo_public_key)?;
    let encrypted_response = depo.handle_request(&encrypted_request).await?;
    let response = encrypted_response.verify_and_decrypt(depo_public_key, client_private_key)?;
    assert_eq!(response.response_id().unwrap(), request.request_id().unwrap());
    Ok(response)
}

#[tokio::test]
async fn test_mem_depot() {
    let depo = Depo::new_in_memory();
    let depo_public_key = depo.public_key();

    // Alice stores a share
    let alice_private_key = PrivateKeyBase::new();
    let alice_public_key = alice_private_key.public_keys();
    let alice_data_1 = Bytes::from_static(&hex!("cafebabe"));
    let request = StoreShareRequest::new(&alice_public_key, alice_data_1.clone());
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let alice_receipt_1 = response.receipt().clone();

    // Bob stores a share
    let bob_private_key = PrivateKeyBase::new();
    let bob_public_key = bob_private_key.public_keys();
    let bob_data_1 = Bytes::from_static(&hex!("deadbeef"));
    let request = StoreShareRequest::new(&bob_public_key, bob_data_1.clone());
    let response_envelope = client_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let bob_receipt_1 = response.receipt().clone();

    // Alice retrieves her share
    let request = GetSharesRequest::new(&alice_public_key, vec![&alice_receipt_1]);
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    let alice_retrieved_data_1 = response.data_for_receipt(&alice_receipt_1).unwrap().clone();
    assert_eq!(alice_retrieved_data_1, alice_data_1);

    // Bob retrieves his share
    let request = GetSharesRequest::new(&bob_public_key, vec![&bob_receipt_1]);
    let response_envelope = client_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    let bob_retrieved_data_1 = response.data_for_receipt(&bob_receipt_1).unwrap().clone();
    assert_eq!(bob_retrieved_data_1, bob_data_1);

    // Alice stores a second share
    let alice_data_2 = Bytes::from_static(&hex!("cafef00d"));
    let request = StoreShareRequest::new(&alice_public_key, alice_data_2.clone());
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let alice_receipt_2 = response.receipt().clone();

    // Alice retrieves her second share
    let request = GetSharesRequest::new(&alice_public_key, vec![&alice_receipt_2]);
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    let alice_retrieved_data_2 = response.data_for_receipt(&alice_receipt_2).unwrap().clone();
    assert_eq!(alice_retrieved_data_2, alice_data_2);

    // Alice retrieves both her shares identified only by her public key
    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    assert_eq!(response.receipt_to_data().len(), 2);

    // Bob attempts to retrieve one of Alice's shares
    let request = GetSharesRequest::new(&bob_public_key, vec![&alice_receipt_1]);
    let response_envelope = client_call(request, &bob_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    assert_eq!(response.receipt_to_data().len(), 0);

    // Someone attempts to retrieve all shares from a nonexistent account
    let nonexistent_private_key = PrivateKeyBase::new();
    let nonexistent_public_key = nonexistent_private_key.public_keys();
    let request = GetSharesRequest::new(&nonexistent_public_key, vec![]);
    let response: Result<Envelope, anyhow::Error> = client_call(request, &nonexistent_private_key, depo_public_key, &depo).await;
    assert_eq!(response.err().unwrap().to_string(), "unknown public key");

    // Someone attempts to retrieve all shares from Alice's account using her public key
    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response: Result<Envelope, anyhow::Error> = client_call(request, &nonexistent_private_key, depo_public_key, &depo).await;
    assert_eq!(response.err().unwrap().to_string(), "could not verify a signature");

    // Alice attempts to retrieve her shares using the incorrect depo public key
    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response: Result<Envelope, anyhow::Error> = client_call(request, &alice_private_key, &nonexistent_public_key, &depo).await;
    assert_eq!(response.err().unwrap().to_string(), "no recipient matches the given key");

    // Alice stores a share she's previously stored (idempotent)
    let request = StoreShareRequest::new(&alice_public_key, alice_data_1.clone());
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = StoreShareResponse::try_from(response_envelope).unwrap();
    let alice_receipt_3 = response.receipt().clone();
    assert_eq!(alice_receipt_3, alice_receipt_1);

    // Alice deletes one of her shares
    let request = DeleteSharesRequest::new(&alice_public_key, vec![&alice_receipt_1]);
    client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();

    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    assert_eq!(response.receipt_to_data().len(), 1);
    let alice_retrieved_data_2 = response.data_for_receipt(&alice_receipt_2).unwrap().clone();
    assert_eq!(alice_retrieved_data_2, alice_data_2);

    // Alice attempts to delete a share she already deleted (idempotent)
    let request = DeleteSharesRequest::new(&alice_public_key, vec![&alice_receipt_1]);
    client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();

    let request = GetSharesRequest::new(&alice_public_key, vec![]);
    let response_envelope = client_call(request, &alice_private_key, depo_public_key, &depo).await.unwrap();
    let response = GetSharesResponse::try_from(response_envelope).unwrap();
    assert_eq!(response.receipt_to_data().len(), 1);
    let alice_retrieved_data_2 = response.data_for_receipt(&alice_receipt_2).unwrap().clone();
    assert_eq!(alice_retrieved_data_2, alice_data_2);

    // // Bob adds a recovery method
    // let bob_recovery = "bob@example.com";
    // depo.update_recovery(&bob_public_key, Some(bob_recovery)).await.unwrap();
    // assert_eq!(depo.get_recovery(&bob_public_key).await.unwrap(), Some(bob_recovery.to_string()));

    // // Alice attempts to add a non-unique recovery method
    // assert!(depo.update_recovery(&alice_public_key, Some(bob_recovery)).await.is_err());
    // assert_eq!(depo.get_recovery(&alice_public_key).await.unwrap(), None);

    // // Someone attempts to retrieve the fallback for a nonexistent account
    // let nonexistent_public_key = PrivateKeyBase::new().public_keys();
    // assert!(depo.get_recovery(&nonexistent_public_key).await.is_err());

    // // Alice updates her public key to a new one
    // let alice_public_key_2 = PrivateKeyBase::new().public_keys();
    // depo.update_key(&alice_public_key, &alice_public_key_2).await.unwrap();

    // // Alice can no longer retrieve her shares using the old public key
    // assert!(depo.get_shares(&alice_public_key, &HashSet::new()).await.is_err());

    // // Alice must now use her new public key
    // let alice_shares = depo.get_shares(&alice_public_key_2, &HashSet::new()).await.unwrap();
    // assert_eq!(alice_shares.len(), 1);

    // // Bob has lost his public key, so he wants to replace it with a new one
    // let bob_public_key_2 = PrivateKeyBase::new().public_keys();

    // // Bob requests transfer using an incorrect recovery method
    // assert!(depo.start_recovery("wrong@example.com", &bob_public_key_2).await.is_err());

    // // Bob requests a transfer using the correct recovery method
    // //
    // // The recovery continuation is sent to Bob's recovery contact method. It is both signed
    // // by the server and encrypted to the server, and is also time-limited.
    // let recovery_continuation = depo.start_recovery(bob_recovery, &bob_public_key_2).await.unwrap();

    // // Bob uses the recovery continuation to finish setting his new public key
    // depo.finish_recovery(&recovery_continuation).await.unwrap();

    // // Bob can no longer retrieve his shares using the old public key
    // assert!(depo.get_shares(&bob_public_key, &HashSet::new()).await.is_err());

    // // Bob must now use his new public key
    // let bob_shares = depo.get_shares(&bob_public_key_2, &HashSet::new()).await.unwrap();
    // assert_eq!(bob_shares.len(), 1);

    // // Bob decides to delete his account
    // depo.delete_account(&bob_public_key_2).await.unwrap();

    // // Bob can no longer retrieve his shares using the new public key
    // assert!(depo.get_shares(&bob_public_key_2, &HashSet::new()).await.is_err());

    // // Attempting to retrieve his fallback now throws an error
    // assert!(depo.get_recovery(&bob_public_key_2).await.is_err());

    // // Deleting an account is idempotent
    // depo.delete_account(&bob_public_key_2).await.unwrap();
}
