use std::collections::{HashSet, HashMap};

use anyhow::bail;
use async_trait::async_trait;
use bc_components::{PublicKeyBase, PrivateKeyBase, ARID};
use tokio::sync::RwLock;

use crate::{local_store::LocalStore, receipt::Receipt, user::User, record::Record};

struct Inner {
    id_to_user: HashMap<ARID, User>,
    fallback_to_id: HashMap<String, ARID>,
    public_key_to_id: HashMap<PublicKeyBase, ARID>,
    receipt_to_record: HashMap<Receipt, Record>,
    id_to_receipts: HashMap<ARID, HashSet<Receipt>>,
}

pub struct LocalStoreMem {
    private_key: PrivateKeyBase,
    public_key: PublicKeyBase,
    inner: RwLock<Inner>,
}

impl LocalStoreMem {
    const MAX_PAYLOAD_SIZE: usize = 1000;
    const CONTINUATION_EXPIRY_SECONDS: f64 = 60.0 * 60.0 * 24.0;

    pub fn new() -> Self {
        let private_key = PrivateKeyBase::new();
        let public_key = private_key.public_keys();
        Self {
            private_key,
            public_key,
            inner: RwLock::new(Inner {
                id_to_user: HashMap::new(),
                fallback_to_id: HashMap::new(),
                public_key_to_id: HashMap::new(),
                receipt_to_record: HashMap::new(),
                id_to_receipts: HashMap::new(),
            })
        }
    }
}

impl Default for LocalStoreMem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LocalStoreMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inner.try_read() {
            Ok(read) => {
                write!(f, "MemStore(users_by_id: {:?}, user_ids_by_public_key: {:?}, records_by_receipt: {:?}), receipts_by_user_id: {:?}",
                    read.id_to_user,
                    read.public_key_to_id,
                    read.receipt_to_record,
                    read.id_to_receipts,
                )
            },
            Err(_) => write!(f, "MemStore: <locked>")
        }
    }
}

#[async_trait]
impl LocalStore for LocalStoreMem {
    fn max_payload_size(&self) -> usize {
        Self::MAX_PAYLOAD_SIZE
    }

    fn continuation_expiry_seconds(&self) -> f64 {
        Self::CONTINUATION_EXPIRY_SECONDS
    }

    fn private_key(&self) -> &PrivateKeyBase {
        &self.private_key
    }

    fn public_key(&self) -> &PublicKeyBase {
        &self.public_key
    }

    async fn existing_public_key_to_id(&self, public_key: &PublicKeyBase) -> anyhow::Result<Option<ARID>> {
        Ok(self.inner.read().await.public_key_to_id.get(public_key).cloned())
    }

    async fn existing_id_to_user(&self, user_id: &ARID) -> anyhow::Result<Option<User>> {
        Ok(self.inner.read().await.id_to_user.get(user_id).cloned())
    }

    async fn insert_user(&self, user: &User) -> anyhow::Result<()> {
        let mut write = self.inner.write().await;
        write.id_to_user.insert(user.user_id().clone(), user.clone());
        write.public_key_to_id.insert(user.public_key().clone(), user.user_id().clone());
        write.id_to_receipts.insert(user.user_id().clone(), HashSet::new());
        Ok(())
    }

    async fn insert_record(&self, record: &Record) -> anyhow::Result<()> {
        let mut write = self.inner.write().await;
        let receipt = record.receipt();
        write.receipt_to_record.insert(receipt.clone(), record.clone());
        write.id_to_receipts.get_mut(record.user_id()).unwrap().insert(receipt.clone());
        Ok(())
    }

    async fn id_to_receipts(&self, user_id: &ARID) -> anyhow::Result<HashSet<Receipt>> {
        Ok(self.inner.read().await.id_to_receipts.get(user_id).unwrap().clone())
    }

    async fn receipt_to_record(&self, receipt: &Receipt) -> anyhow::Result<Record> {
        let read = self.inner.read().await;
        let record = read.receipt_to_record.get(receipt);
        let record = match record {
            Some(record) => record,
            None => bail!("unknown receipt"),
        };
        Ok(record.clone())
    }

    async fn delete_record(&self, receipt: &Receipt) -> anyhow::Result<()> {
        let record = self.receipt_to_record(receipt).await?;
        let mut write = self.inner.write().await;
        write.receipt_to_record.remove(receipt);
        write.id_to_receipts.get_mut(record.user_id()).unwrap().remove(receipt);
        Ok(())
    }

    async fn set_user_public_key(&self, old_public_key: &PublicKeyBase, new_public_key: &PublicKeyBase) -> anyhow::Result<()> {
        let user = self.expect_public_key_to_user(old_public_key).await?;
        let mut write = self.inner.write().await;
        write.public_key_to_id.remove(old_public_key);
        write.public_key_to_id.insert(new_public_key.clone(), user.user_id().clone());
        let user = write.id_to_user.get_mut(user.user_id()).unwrap();
        user.set_public_key(new_public_key.clone());
        Ok(())
    }

    async fn set_user_fallback(&self, user: &User, fallback: Option<&str>) -> anyhow::Result<()> {
        let mut write = self.inner.write().await;

        // get the user's existing fallback
        let old_fallback = user.fallback();
        // if the new and old fallbacks are the same, return (idempotency)
        if old_fallback == fallback {
            return Ok(());
        }
        // Remove the old fallback, if any
        if let Some(old_fallback) = old_fallback {
            write.fallback_to_id.remove(old_fallback);
        }
        // Add the new fallback, if any
        if let Some(fallback) = fallback {
            write.fallback_to_id.insert(fallback.to_string(), user.user_id().clone());
        }
        // Set the user record to the new fallback
        let user = write.id_to_user.get_mut(user.user_id()).unwrap();
        user.set_fallback(fallback);
        Ok(())
    }

    async fn remove_user(&self, user: &User) -> anyhow::Result<()> {
        let mut write = self.inner.write().await;

        write.public_key_to_id.remove(user.public_key());
        write.fallback_to_id.remove(user.fallback().unwrap_or_default());
        write.id_to_user.remove(user.user_id());
        write.id_to_receipts.remove(user.user_id());
        Ok(())
    }

    async fn fallback_to_user(&self, fallback: &str) -> anyhow::Result<Option<User>> {
        let read = self.inner.read().await;
        let user_id = read.fallback_to_id
            .get(fallback)
            .map_or(Err(anyhow::anyhow!("unknown fallback")), Ok)?;
        let user = self.existing_id_to_user(user_id).await?;
        Ok(user)
    }
}
