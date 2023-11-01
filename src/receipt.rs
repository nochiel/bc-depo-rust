use bc_components::{ARID, Digest};
use bc_envelope::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Receipt(Digest);

impl Receipt {
    pub fn new(user: &ARID, payload: impl AsRef<[u8]>) -> Self {
        Self(Digest::from_image_parts(&[user.data(), payload.as_ref()]))
    }
}

impl std::fmt::Debug for Receipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Receipt({})", hex::encode(&self.0))
    }
}

impl EnvelopeEncodable for Receipt {
    fn envelope(self) -> Envelope {
        Envelope::new(CBOR::byte_string(self.0))
            .add_type("receipt")
    }
}

impl From<Receipt> for Envelope {
    fn from(receipt: Receipt) -> Self {
        receipt.envelope()
    }
}

impl EnvelopeDecodable for Receipt {
    fn from_envelope(envelope: Envelope) -> anyhow::Result<Self> {
        envelope.clone().check_type_envelope("receipt")?;
        let cbor: CBOR = envelope.extract_subject()?;
        let bytes = cbor.expect_byte_string()?;
        let digest = Digest::from_data_ref(&bytes)?;
        Ok(Self(digest))
    }
}

impl TryFrom<Envelope> for Receipt {
    type Error = anyhow::Error;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Self::from_envelope(envelope)
    }
}

impl EnvelopeCodable for Receipt { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt() {
        let receipt = Receipt::new(&ARID::new(), b"payload");
        let envelope = receipt.clone().envelope();
        let receipt_2 = Receipt::from_envelope(envelope.clone()).unwrap();
        assert_eq!(receipt, receipt_2);
        println!("{:?}", receipt);
        println!("{}", envelope.ur_string());
        println!("{}", envelope.format());
    }
}
