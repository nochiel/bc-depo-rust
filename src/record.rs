use std::fmt::Formatter;

use bc_components::ARID;
use bytes::Bytes;

use crate::receipt::Receipt;

// #[derive(Debug)]
pub struct Record {
    // The userID is for internal use only, and never changes for a given account.
    // Users always identify themselves by a public key, which can change over the
    // lifetime of the account.
    user_id: ARID,
    payload: Bytes,
    receipt: Receipt,
}

impl Record {
    pub fn new(user_id: ARID, payload: Bytes) -> Self {
        let receipt = Receipt::new(&user_id, &payload);
        Self {
            user_id,
            payload,
            receipt,
        }
    }

    pub fn user_id(&self) -> &ARID {
        &self.user_id
    }

    pub fn payload(&self) -> &Bytes {
        &self.payload
    }

    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }
}

struct HexBytes(Bytes);

impl HexBytes {
    fn new(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

impl std::fmt::Debug for HexBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bytes({})", hex::encode(&self.0))
    }
}

impl std::fmt::Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Record")
            .field("user_id", &self.user_id)
            .field("payload", &HexBytes::new(self.payload.clone()))
            .field("receipt", &self.receipt)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let record = Record::new(ARID::new(), Bytes::from_static(&[0x01, 0x02, 0x03]));
        println!("{:?}", record);
    }
}
