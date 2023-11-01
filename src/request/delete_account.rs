use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::DepoRequest;

#[derive(Debug, Clone)]
pub struct DeleteAccount {
    key: PublicKeyBase,
}

impl DeleteAccount {
    pub fn new(key: PublicKeyBase) -> Self {
        Self {
            key,
        }
    }
}

impl EnvelopeEncodable for DeleteAccount {
    fn envelope(self) -> Envelope {
        Envelope::new_function(Self::function())
            .add_parameter(Self::key_param(), self.key)
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
        let public_key: PublicKeyBase = envelope.extract_object_for_parameter(Self::key_param())?;
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
    fn key(&self) -> &PublicKeyBase {
        &self.key
    }
}
