use std::collections::HashSet;

use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use crate::receipt::Receipt;

use super::DepoRequest;

#[derive(Debug, Clone)]
pub struct GetShares {
    key: PublicKeyBase,
    receipts: HashSet<Receipt>,
}

impl GetShares {
    pub fn new(key: PublicKeyBase, receipts: HashSet<Receipt>) -> Self {
        Self {
            key,
            receipts,
        }
    }

    pub fn receipts(&self) -> &HashSet<Receipt> {
        &self.receipts
    }

    fn receipt_param() -> Parameter {
        Parameter::new_named("receipt")
    }
}

impl EnvelopeEncodable for GetShares {
    fn envelope(self) -> Envelope {
        let mut e = Envelope::new_function(Self::function())
            .add_parameter(Self::key_param(), self.key);

        for receipt in self.receipts {
            e = e.add_parameter(Self::receipt_param(), receipt);
        }

        e
    }
}

impl From<GetShares> for Envelope {
    fn from(request: GetShares) -> Self {
        request.envelope()
    }
}

impl EnvelopeDecodable for GetShares {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.check_function(&Self::function())?;
        let public_key: PublicKeyBase = envelope.extract_object_for_parameter(Self::key_param())?;
        let receipts = envelope.objects_for_parameter(Self::receipt_param())
            .into_iter()
            .map(|e| e.try_into())
            .collect::<anyhow::Result<HashSet<Receipt>>>()?;
        Ok(Self::new(public_key, receipts))
    }
}

impl TryFrom<Envelope> for GetShares {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for GetShares {}

impl RequestBody for GetShares {
    fn function() -> Function {
        Function::new_named("getShares")
    }
}

impl DepoRequest for GetShares {
    fn key(&self) -> &PublicKeyBase {
        &self.key
    }
}
