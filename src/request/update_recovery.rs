use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::DepoRequest;

#[derive(Debug, Clone)]
pub struct UpdateRecovery {
    key: PublicKeyBase,
    recovery: Option<String>,
}

impl UpdateRecovery {
    pub fn new(key: PublicKeyBase, recovery: Option<impl AsRef<str>>) -> Self {
        Self {
            key,
            recovery: recovery.map(|s| s.as_ref().to_string()),
        }
    }

    pub fn recovery(&self) -> Option<&str> {
        self.recovery.as_deref()
    }
}

impl EnvelopeEncodable for UpdateRecovery {
    fn envelope(self) -> Envelope {
        Envelope::new_function(Self::function())
            .add_parameter(Self::key_param(), self.key)
            .add_optional_parameter("recovery", self.recovery)
    }
}

impl From<UpdateRecovery> for Envelope {
    fn from(request: UpdateRecovery) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for UpdateRecovery {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let key: PublicKeyBase = envelope.extract_object_for_parameter(Self::key_param())?;
        let recovery: Option<String> = envelope.extract_optional_object_for_parameter("recovery")?;
        Ok(Self::new(key, recovery))
    }
}

impl TryFrom<Envelope> for UpdateRecovery {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for UpdateRecovery {}

impl RequestBody for UpdateRecovery {
    fn function() -> Function {
        Function::new_named("updateRecovery")
    }
}

impl DepoRequest for UpdateRecovery {
    fn key(&self) -> &PublicKeyBase {
        &self.key
    }
}
