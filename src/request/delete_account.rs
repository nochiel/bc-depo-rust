use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::depo_request::DepoRequest;

#[derive(Debug, Clone)]
pub struct DeleteAccount {
    public_key: PublicKeyBase,
}

impl DeleteAccount {
    pub fn new(public_key: PublicKeyBase) -> Self {
        Self {
            public_key,
        }
    }
}

impl EnvelopeEncodable for DeleteAccount {
    fn envelope(self) -> Envelope {
        Envelope::new_function("storeShare")
            .add_parameter("publicKey", self.public_key)
    }
}

impl From<DeleteAccount> for Envelope {
    fn from(request: DeleteAccount) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for DeleteAccount {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let public_key: PublicKeyBase = envelope.extract_object_for_parameter("publicKey")?;
        Ok(Self::new(public_key))
    }
}

impl TryFrom<Envelope> for DeleteAccount {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for DeleteAccount {}

impl RequestBody for DeleteAccount {
    fn function() -> Function {
        Function::new_named("deleteAccount")
    }
}

impl DepoRequest for DeleteAccount {
    fn public_key(&self) -> &PublicKeyBase {
        &self.public_key
    }
}
