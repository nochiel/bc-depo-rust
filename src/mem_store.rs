use std::{collections::{HashSet, HashMap}, time::Duration};

use async_trait::async_trait;
use bc_components::{PublicKeyBase, PrivateKeyBase, ARID};
use bytes::Bytes;

use crate::{store::Store, receipt::Receipt, user::User, record::Record};

pub struct MemStore {
    private_key: PrivateKeyBase,
    public_key: PublicKeyBase,
    users_by_id: HashMap<ARID, User>,
    user_ids_by_fallback: HashMap<String, ARID>,
    user_ids_by_public_key: HashMap<PublicKeyBase, ARID>,
    records_by_receipt: HashMap<Receipt, Record>,
    receipts_by_user_id: HashMap<ARID, HashSet<Receipt>>,
}

impl MemStore {
    const MAX_PAYLOAD_SIZE: usize = 1000;
    const REQUEST_EXPIRY: Duration = Duration::from_secs(10);

    pub fn new() -> Self {
        let private_key = PrivateKeyBase::new();
        let public_key = private_key.public_keys();
        Self {
            private_key,
            public_key,
            users_by_id: HashMap::new(),
            user_ids_by_fallback: HashMap::new(),
            user_ids_by_public_key: HashMap::new(),
            records_by_receipt: HashMap::new(),
            receipts_by_user_id: HashMap::new(),
        }
    }
}

impl Default for MemStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MemStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemStore(users_by_id: {:?}, user_ids_by_public_key: {:?}, records_by_receipt: {:?}), receipts_by_user_id: {:?}",
            self.users_by_id,
            self.user_ids_by_public_key,
            self.records_by_receipt,
            self.receipts_by_user_id,
        )
    }
}

#[async_trait]
impl Store for MemStore {
    async fn store(&self, public_key: &PublicKeyBase, payload: Bytes) -> anyhow::Result<Receipt> {
        todo!();
    }

    async fn update_fallback(&self, public_key: &PublicKeyBase, fallback: Option<&str>) -> anyhow::Result<()> {
        todo!();
    }

    async fn get_fallback(&self, public_key: &PublicKeyBase) -> anyhow::Result<Option<String>> {
        todo!();
    }

    async fn change_public_key(&self, old_public_key: &PublicKeyBase, new_public_key: &PublicKeyBase) -> anyhow::Result<()> {
        todo!();
    }

    async fn delete(&self, public_key: &PublicKeyBase, receipts: &HashSet<Receipt>) -> anyhow::Result<()> {
        todo!();
    }

    async fn retrieve(&self, public_key: &PublicKeyBase, receipts: &HashSet<Receipt>) -> anyhow::Result<HashSet<(Receipt, Bytes)>> {
        todo!();
    }

    async fn delete_account(&self, public_key: &PublicKeyBase) -> anyhow::Result<()> {
        todo!();
    }

    async fn request_reset(&self, fallback: &str, new_public_key: &PublicKeyBase) -> anyhow::Result<()> {
        todo!();
    }
}
