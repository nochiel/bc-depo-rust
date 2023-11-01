use std::collections::{HashSet, HashMap};

use async_trait::async_trait;
use bc_components::{PublicKeyBase, PrivateKeyBase, ARID};
use tokio::sync::RwLock;

use crate::{depo_impl::DepoImpl, receipt::Receipt, user::User, record::Record, depo::Depo};

struct Inner {
    id_to_user: HashMap<ARID, User>,
    recovery_to_id: HashMap<String, ARID>,
    public_key_to_id: HashMap<PublicKeyBase, ARID>,
    receipt_to_record: HashMap<Receipt, Record>,
    id_to_receipts: HashMap<ARID, HashSet<Receipt>>,
}

struct MemDepoImpl {
    private_key: PrivateKeyBase,
    public_key: PublicKeyBase,
    inner: RwLock<Inner>,
}

impl MemDepoImpl {
    const MAX_PAYLOAD_SIZE: usize = 1000;
    const CONTINUATION_EXPIRY_SECONDS: f64 = 60.0 * 60.0 * 24.0;

    fn new() -> Box<Self> {
        let private_key = PrivateKeyBase::new();
        let public_key = private_key.public_keys();
        Box::new(Self {
            private_key,
            public_key,
            inner: RwLock::new(Inner {
                id_to_user: HashMap::new(),
                recovery_to_id: HashMap::new(),
                public_key_to_id: HashMap::new(),
                receipt_to_record: HashMap::new(),
                id_to_receipts: HashMap::new(),
            })
        })
    }
}

impl std::fmt::Debug for MemDepoImpl {
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
impl DepoImpl for MemDepoImpl {
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

    async fn existing_key_to_id(&self, public_key: &PublicKeyBase) -> anyhow::Result<Option<ARID>> {
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

    async fn receipt_to_record(&self, receipt: &Receipt) -> anyhow::Result<Option<Record>> {
        let read = self.inner.read().await;
        let record = read.receipt_to_record.get(receipt);
        Ok(record.cloned())
    }

    async fn delete_record(&self, receipt: &Receipt) -> anyhow::Result<()> {
        let record = self.receipt_to_record(receipt).await?;
        if let Some(record) = record {
            let mut write = self.inner.write().await;
            write.receipt_to_record.remove(receipt);
            write.id_to_receipts.get_mut(record.user_id()).unwrap().remove(receipt);
        }
        Ok(())
    }

    async fn set_user_key(&self, old_public_key: &PublicKeyBase, new_public_key: &PublicKeyBase) -> anyhow::Result<()> {
        let user = self.expect_key_to_user(old_public_key).await?;
        let mut write = self.inner.write().await;
        write.public_key_to_id.remove(old_public_key);
        write.public_key_to_id.insert(new_public_key.clone(), user.user_id().clone());
        let user = write.id_to_user.get_mut(user.user_id()).unwrap();
        user.set_public_key(new_public_key.clone());
        Ok(())
    }

    async fn set_user_recovery(&self, user: &User, recovery: Option<&str>) -> anyhow::Result<()> {
        let mut write = self.inner.write().await;

        // get the user's existing recovery
        let old_recovery = user.recovery();
        // if the new and old recoverys are the same, return (idempotency)
        if old_recovery == recovery {
            return Ok(());
        }
        // Remove the old recovery, if any
        if let Some(old_recovery) = old_recovery {
            write.recovery_to_id.remove(old_recovery);
        }
        // Add the new recovery, if any
        if let Some(recovery) = recovery {
            write.recovery_to_id.insert(recovery.to_string(), user.user_id().clone());
        }
        // Set the user record to the new recovery
        let user = write.id_to_user.get_mut(user.user_id()).unwrap();
        user.set_recovery(recovery);
        Ok(())
    }

    async fn remove_user(&self, user: &User) -> anyhow::Result<()> {
        let mut write = self.inner.write().await;

        write.public_key_to_id.remove(user.public_key());
        write.recovery_to_id.remove(user.recovery().unwrap_or_default());
        write.id_to_user.remove(user.user_id());
        write.id_to_receipts.remove(user.user_id());
        Ok(())
    }

    async fn recovery_to_user(&self, recovery: &str) -> anyhow::Result<Option<User>> {
        let read = self.inner.read().await;
        let user_id = read.recovery_to_id
            .get(recovery);
        let user = if let Some(user_id) = user_id {
            self.existing_id_to_user(user_id).await?
        } else {
            None
        };
        Ok(user)
    }
}

impl Depo {
    pub fn new_in_memory() -> Self {
        Self::new(MemDepoImpl::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use hex_literal::hex;

    #[tokio::test]
    async fn test_mem_depot() {
        let depo = Depo::new_in_memory();

        // Alice stores a share
        let alice_public_key = PrivateKeyBase::new().public_keys();
        let alice_data_1 = Bytes::from_static(&hex!("cafebabe"));
        let alice_receipt1 = depo.store_share(&alice_public_key, &alice_data_1).await.unwrap();

        // Bob stores a share
        let bob_public_key = PrivateKeyBase::new().public_keys();
        let bob_data_1 = Bytes::from_static(&hex!("deadbeef"));
        let bob_receipt1 = depo.store_share(&bob_public_key, &bob_data_1).await.unwrap();

        // Alice retrieves her share
        assert_eq!(depo.get_share(&alice_public_key, &alice_receipt1).await.unwrap(), alice_data_1);

        // Bob retrieves his share
        assert_eq!(depo.get_share(&bob_public_key, &bob_receipt1).await.unwrap(), bob_data_1);

        // Alice stores a second share
        let alice_data_2 = Bytes::from_static(&hex!("cafef00d"));
        let alice_receipt_2 = depo.store_share(&alice_public_key, &alice_data_2).await.unwrap();

        // Alice retrieves her second share
        assert_eq!(depo.get_share(&alice_public_key, &alice_receipt_2).await.unwrap(), alice_data_2);

        // Alice retrieves both her shares identified only by her public key
        let alice_shares = depo.get_shares(&alice_public_key, &HashSet::new()).await.unwrap();
        assert_eq!(alice_shares.len(), 2);

        // Bob attempts to retrieve one of Alice's shares
        assert!(depo.get_share(&bob_public_key, &alice_receipt1).await.is_err());

        // Someone attempts to retrieve all shares from a nonexistent account
        let nonexistent_public_key = PrivateKeyBase::new().public_keys();
        assert!(depo.get_shares(&nonexistent_public_key, &HashSet::new()).await.is_err());

        // Alice stores a share she's previously stored (idempotent)
        let alice_receipt_3 = depo.store_share(&alice_public_key, &alice_data_1).await.unwrap();
        assert_eq!(alice_receipt1, alice_receipt_3);

        // Alice deletes one of her shares
        depo.delete_share(&alice_public_key, &alice_receipt1).await.unwrap();
        let alice_shares = depo.get_shares(&alice_public_key, &HashSet::new()).await.unwrap();
        assert_eq!(alice_shares.len(), 1);
        assert_eq!(alice_shares.iter().next().unwrap().1, &alice_data_2);

        // Alice attempts to delete a share she already deleted (idempotent)
        depo.delete_share(&alice_public_key, &alice_receipt1).await.unwrap();
        let alice_shares = depo.get_shares(&alice_public_key, &HashSet::new()).await.unwrap();
        assert_eq!(alice_shares.len(), 1);
        assert_eq!(alice_shares.iter().next().unwrap().1, &alice_data_2);

        // Bob adds a recovery method
        let bob_recovery = "bob@example.com";
        depo.update_recovery(&bob_public_key, Some(bob_recovery)).await.unwrap();
        assert_eq!(depo.get_recovery(&bob_public_key).await.unwrap(), Some(bob_recovery.to_string()));

        // Alice attempts to add a non-unique recovery method
        assert!(depo.update_recovery(&alice_public_key, Some(bob_recovery)).await.is_err());
        assert_eq!(depo.get_recovery(&alice_public_key).await.unwrap(), None);

        // Someone attempts to retrieve the fallback for a nonexistent account
        let nonexistent_public_key = PrivateKeyBase::new().public_keys();
        assert!(depo.get_recovery(&nonexistent_public_key).await.is_err());

        // Alice updates her public key to a new one
        let alice_public_key_2 = PrivateKeyBase::new().public_keys();
        depo.update_key(&alice_public_key, &alice_public_key_2).await.unwrap();

        // Alice can no longer retrieve her shares using the old public key
        assert!(depo.get_shares(&alice_public_key, &HashSet::new()).await.is_err());

        // Alice must now use her new public key
        let alice_shares = depo.get_shares(&alice_public_key_2, &HashSet::new()).await.unwrap();
        assert_eq!(alice_shares.len(), 1);

        // Bob has lost his public key, so he wants to replace it with a new one
        let bob_public_key_2 = PrivateKeyBase::new().public_keys();

        // Bob requests transfer using an incorrect recovery method
        assert!(depo.start_recovery_transfer("wrong@example.com", &bob_public_key_2).await.is_err());

        // Bob requests a transfer using the correct recovery method
        //
        // The recovery continuation is sent to Bob's recovery contact method. It is both signed
        // by the server and encrypted to the server, and is also time-limited.
        let recovery_continuation = depo.start_recovery_transfer(bob_recovery, &bob_public_key_2).await.unwrap();

        // Bob uses the recovery continuation to finish setting his new public key
        depo.finish_recovery_transfer(&recovery_continuation).await.unwrap();

        // Bob can no longer retrieve his shares using the old public key
        assert!(depo.get_shares(&bob_public_key, &HashSet::new()).await.is_err());

        // Bob must now use his new public key
        let bob_shares = depo.get_shares(&bob_public_key_2, &HashSet::new()).await.unwrap();
        assert_eq!(bob_shares.len(), 1);

        // Bob decides to delete his account
        depo.delete_account(&bob_public_key_2).await.unwrap();

        // Bob can no longer retrieve his shares using the new public key
        assert!(depo.get_shares(&bob_public_key_2, &HashSet::new()).await.is_err());

        // Attempting to retrieve his fallback now throws an error
        assert!(depo.get_recovery(&bob_public_key_2).await.is_err());

        // Deleting an account is idempotent
        depo.delete_account(&bob_public_key_2).await.unwrap();
    }
}
