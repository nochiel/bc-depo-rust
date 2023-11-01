use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::DepoRequest;

#[derive(Debug, Clone)]
pub struct UpdateFallback {
    key: PublicKeyBase,
    fallback: Option<String>,
}

impl UpdateFallback {
    pub fn new(key: PublicKeyBase, fallback: Option<impl AsRef<str>>) -> Self {
        Self {
            key,
            fallback: fallback.map(|s| s.as_ref().to_string()),
        }
    }

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }
}

impl EnvelopeEncodable for UpdateFallback {
    fn envelope(self) -> Envelope {
        Envelope::new_function(Self::function())
            .add_parameter(Self::key_param(), self.key)
            .add_optional_parameter("fallback", self.fallback)
    }
}

impl From<UpdateFallback> for Envelope {
    fn from(request: UpdateFallback) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for UpdateFallback {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let key: PublicKeyBase = envelope.extract_object_for_parameter(Self::key_param())?;
        let fallback: Option<String> = envelope.extract_optional_object_for_parameter("fallback")?;
        Ok(Self::new(key, fallback))
    }
}

impl TryFrom<Envelope> for UpdateFallback {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for UpdateFallback {}

impl RequestBody for UpdateFallback {
    fn function() -> Function {
        Function::new_named("updateFallback")
    }
}

impl DepoRequest for UpdateFallback {
    fn key(&self) -> &PublicKeyBase {
        &self.key
    }
}
