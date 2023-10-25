use std::rc::Rc;

use bc_components::{ARID, Digest};
use bc_envelope::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Receipt(Vec<u8>);

impl Receipt {
    pub fn new(user: &ARID, payload: &[u8]) -> Self {
        Self(Digest::from_image_parts(&[user.data(), payload]).data().to_vec())
    }
}

impl std::fmt::Debug for Receipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Receipt({})", hex::encode(&self.0))
    }
}

impl IntoEnvelope for &Receipt {
    fn into_envelope(self) -> Rc<Envelope> {
        Envelope::new(CBOR::byte_string(&self.0))
            .add_type("receipt")
    }
}

impl FromEnvelope for Receipt {
    fn from_envelope(envelope: Rc<Envelope>) -> anyhow::Result<Self> {
        envelope.clone().check_type_envelope("receipt")?;
        let cbor: Rc<CBOR> = envelope.extract_subject()?;
        let bytes = cbor.expect_byte_string()?;
        Ok(Self(bytes.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt() {
        let receipt = Receipt::new(&ARID::new(), b"payload");
        let envelope = receipt.into_envelope();
        let receipt_2 = Receipt::from_envelope(envelope).unwrap();
        assert_eq!(receipt, receipt_2);
        println!("{:?}", receipt);
        // println!("{}", envelope.ur_string());
        // with_format_context!(|context| {
        //     println!("{}", envelope.format_opt(Some(context)));
        // });
    }
}
