use std::fmt::Formatter;

use bc_components::ARID;
use bytes::Bytes;

use depo_api::receipt::Receipt;

#[derive(Clone)]
pub struct Record {
    // The userID is for internal use only, and never changes for a given account.
    // Users always identify themselves by a public key, which can change over the
    // lifetime of the account.
    receipt: Receipt,
    user_id: ARID,
    data: Bytes,
}

impl Record {
    pub fn new(user_id: &ARID, data: &Bytes) -> Self {
        Self::new_opt(Receipt::new(user_id, data), user_id.clone(), data.clone())
    }

    pub fn new_opt(receipt: Receipt, user_id: ARID, data: Bytes) -> Self {
        Self {
            receipt,
            user_id,
            data,
        }
    }

    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }

    pub fn user_id(&self) -> &ARID {
        &self.user_id
    }

    pub fn data(&self) -> &Bytes {
        &self.data
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
            .field("data", &HexBytes::new(self.data.clone()))
            .field("receipt", &self.receipt)
            .finish()
    }
}
