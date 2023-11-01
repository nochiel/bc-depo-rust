use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;
use bytes::Bytes;

use super::depo_request::DepoRequest;

#[derive(Debug, Clone)]
pub struct StoreShare {
    public_key: PublicKeyBase,
    payload: Bytes,
}

impl StoreShare {
    pub fn new(public_key: PublicKeyBase, payload: Bytes) -> Self {
        Self {
            public_key,
            payload,
        }
    }
}

impl EnvelopeEncodable for StoreShare {
    fn envelope(self) -> Envelope {
        Envelope::new_function("storeShare")
            .add_parameter("publicKey", self.public_key)
            .add_parameter("payload", self.payload)
    }
}

impl From<StoreShare> for Envelope {
    fn from(request: StoreShare) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for StoreShare {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let public_key: PublicKeyBase = envelope.extract_object_for_parameter("publicKey")?;
        let payload = envelope.extract_object_for_parameter("payload")?;
        Ok(Self::new(public_key, payload))
    }
}

impl TryFrom<Envelope> for StoreShare {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for StoreShare {}

impl RequestBody for StoreShare {
    fn function() -> Function {
        Function::new_named("storeShare")
    }
}

impl DepoRequest for StoreShare {
    fn public_key(&self) -> &PublicKeyBase {
        &self.public_key
    }
}

#[cfg(test)]
mod tests {
    use bc_components::{PrivateKeyBase, ARID};

    use super::*;
    use indoc::indoc;

    #[test]
    fn test_store_share_request() {
        let client_private_key = PrivateKeyBase::new();
        let client_public_key = client_private_key.public_keys();
        let payload = Bytes::from_static(b"payload");
        let request_body = StoreShare::new(client_public_key.clone(), payload);
        let envelope = request_body.clone().envelope();
        assert_eq!(envelope.format(),
            indoc! {r#"
            «"storeShare"» [
                ❰"payload"❱: Bytes(7)
                ❰"publicKey"❱: PublicKeyBase
            ]
            "#}.trim()
        );

        let id = ARID::from_data_ref(hex_literal::hex!("8712dfac3d0ebfa910736b2a9ee39d4b68f64222a77bcc0074f3f5f1c9216d30")).unwrap();
        let date = dcbor::Date::new_from_string("2023-10-28T07:59:43Z").unwrap();
        let request = Request::new(Some(id), request_body, "This is the note.", Some(date));
        assert_eq!(request.clone().envelope().format(),
            indoc! {r#"
            request(ARID(8712dfac)) [
                'body': «"storeShare"» [
                    ❰"payload"❱: Bytes(7)
                    ❰"publicKey"❱: PublicKeyBase
                ]
                'date': 2023-10-28T07:59:43Z
                'note': "This is the note."
            ]
            "#}.trim()
        );

        let server_private_key = PrivateKeyBase::new();
        let server_public_key = server_private_key.public_keys();

        let encrypted_request = request.clone().envelope().sign_and_encrypt(&client_private_key, &server_public_key).unwrap();
        assert_eq!(encrypted_request.format(),
            indoc! {r#"
            ENCRYPTED [
                'hasRecipient': SealedMessage
                'verifiedBy': Signature
            ]
            "#}.trim()
        );

        let decrypted_request = encrypted_request.verify_and_decrypt(&client_public_key, &server_private_key).unwrap();
        assert_eq!(decrypted_request.format(),
            indoc! {r#"
            request(ARID(8712dfac)) [
                'body': «"storeShare"» [
                    ❰"payload"❱: Bytes(7)
                    ❰"publicKey"❱: PublicKeyBase
                ]
                'date': 2023-10-28T07:59:43Z
                'note': "This is the note."
            ]
            "#}.trim()
        );

        let received_request = Request::<StoreShare>::from_envelope(decrypted_request).unwrap();
        assert!(received_request.envelope().is_identical_to(request.envelope()));
    }
}
