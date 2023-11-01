use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use super::depo_request::DepoRequest;

#[derive(Debug, Clone)]
pub struct UpdateKey {
    old_key: PublicKeyBase,
    new_key: PublicKeyBase,
}

impl UpdateKey {
    pub fn new(old_key: PublicKeyBase, new_key: PublicKeyBase) -> Self {
        Self {
            old_key,
            new_key,
        }
    }

    pub fn new_key(&self) -> &PublicKeyBase {
        &self.new_key
    }
}

impl EnvelopeEncodable for UpdateKey {
    fn envelope(self) -> Envelope {
        Envelope::new_function("storeShare")
            .add_parameter("old", self.old_key)
            .add_parameter("new", self.new_key)
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
        let old_key: PublicKeyBase = envelope.extract_object_for_parameter("old")?;
        let new_key: PublicKeyBase = envelope.extract_object_for_parameter("new")?;
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
    fn public_key(&self) -> &PublicKeyBase {
        &self.old_key
    }
}
