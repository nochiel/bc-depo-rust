use std::collections::HashSet;

use bc_components::PublicKeyBase;
use bc_envelope::prelude::*;

use crate::receipt::Receipt;

use super::depo_request::DepoRequest;

#[derive(Debug, Clone)]
pub struct GetShares {
    public_key: PublicKeyBase,
    receipts: HashSet<Receipt>,
}

impl GetShares {
    pub fn new(public_key: PublicKeyBase, receipts: HashSet<Receipt>) -> Self {
        Self {
            public_key,
            receipts,
        }
    }

    pub fn receipts(&self) -> &HashSet<Receipt> {
        &self.receipts
    }
}

impl EnvelopeEncodable for GetShares {
    fn envelope(self) -> Envelope {
        let mut e = Envelope::new_function("storeShare")
            .add_parameter("publicKey", self.public_key);

        for receipt in self.receipts {
            e = e.add_parameter("receipt", receipt);
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
        let public_key: PublicKeyBase = envelope.extract_object_for_parameter("publicKey")?;
        let receipts = envelope.objects_for_parameter("receipt")
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
    fn public_key(&self) -> &PublicKeyBase {
        &self.public_key
    }
}
