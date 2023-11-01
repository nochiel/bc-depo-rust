use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::DepoRequest;

#[derive(Debug, Clone)]
pub struct UpdateKey {
    key: PublicKeyBase,
    new_key: PublicKeyBase,
}

impl UpdateKey {
    pub fn new(key: PublicKeyBase, new_key: PublicKeyBase) -> Self {
        Self {
            key,
            new_key,
        }
    }

    pub fn new_key(&self) -> &PublicKeyBase {
        &self.new_key
    }

    fn new_key_param() -> Parameter {
        Parameter::new_named("newKey")
    }
}

impl EnvelopeEncodable for UpdateKey {
    fn envelope(self) -> Envelope {
        Envelope::new_function(Self::function())
            .add_parameter(Self::key_param(), self.key)
            .add_parameter(Self::new_key_param(), self.new_key)
    }
}

impl From<UpdateKey> for Envelope {
    fn from(request: UpdateKey) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for UpdateKey {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let old_key: PublicKeyBase = envelope.extract_object_for_parameter(Self::key_param())?;
        let new_key: PublicKeyBase = envelope.extract_object_for_parameter(Self::new_key_param())?;
        Ok(Self::new(old_key, new_key))
    }
}

impl TryFrom<Envelope> for UpdateKey {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for UpdateKey {}

impl RequestBody for UpdateKey {
    fn function() -> Function {
        Function::new_named("updateKey")
    }
}

impl DepoRequest for UpdateKey {
    fn key(&self) -> &PublicKeyBase {
        &self.key
    }
}
