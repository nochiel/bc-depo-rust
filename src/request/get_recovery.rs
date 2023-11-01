use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::DepoRequest;

#[derive(Debug, Clone)]
pub struct GetRecovery {
    key: PublicKeyBase,
}

impl GetRecovery {
    pub fn new(key: PublicKeyBase) -> Self {
        Self {
            key,
        }
    }
}

impl EnvelopeEncodable for GetRecovery {
    fn envelope(self) -> Envelope {
        Envelope::new_function(Self::function())
            .add_parameter(Self::key_param(), self.key)
    }
}

impl From<GetRecovery> for Envelope {
    fn from(request: GetRecovery) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for GetRecovery {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let public_key: PublicKeyBase = envelope.extract_object_for_parameter(Self::key_param())?;
        Ok(Self::new(public_key))
    }
}

impl TryFrom<Envelope> for GetRecovery {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for GetRecovery {}

impl RequestBody for GetRecovery {
    fn function() -> Function {
        Function::new_named("getRecovery")
    }
}

impl DepoRequest for GetRecovery {
    fn key(&self) -> &PublicKeyBase {
        &self.key
    }
}
