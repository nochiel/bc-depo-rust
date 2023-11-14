use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use bc_components::{PublicKeyBase, PrivateKeyBase, ARID};
use depo_api::receipt::Receipt;
use mysql_async::Pool;
use bc_envelope::prelude::*;

use crate::{depo_impl::DepoImpl, user::User, record::Record, depo_struct::Depo, db::{get_settings, db_pool}};

struct DbDepoImpl {
    pool: Pool,
    private_key: PrivateKeyBase,
    public_key: PublicKeyBase,
    public_key_string: String,
    continuation_expiry_seconds: u32,
    max_payload_size: u32,
}

impl DbDepoImpl {
    async fn new() -> anyhow::Result<Arc<Self>> {
        let pool = db_pool();
        let (private_key, continuation_expiry_seconds, max_payload_size) = get_settings(&pool).await?;
        let public_key = private_key.public_keys();
        let public_key_string = public_key.ur_string();
        Ok(Arc::new(Self {
            pool,
            private_key,
            public_key,
            public_key_string,
            continuation_expiry_seconds,
            max_payload_size,
        }))
    }
}

#[async_trait]
impl DepoImpl for DbDepoImpl {
    fn max_payload_size(&self) -> u32 {
        self.max_payload_size
    }

    fn continuation_expiry_seconds(&self) -> u32 {
        self.continuation_expiry_seconds
    }

    fn private_key(&self) ->  &PrivateKeyBase {
        &self.private_key
    }

    fn public_key(&self) ->  &PublicKeyBase {
        &self.public_key
    }

    fn public_key_string(&self) -> &str {
        &self.public_key_string
    }

    async fn existing_key_to_id(&self, public_key: &PublicKeyBase) -> anyhow::Result<Option<ARID>> {
        todo!()
    }

    async fn existing_id_to_user(&self, user_id: &ARID) -> anyhow::Result<Option<User>> {
        todo!()
    }

    async fn insert_user(&self, user: &User) -> anyhow::Result<()> {
        todo!()
    }

    async fn insert_record(&self, record: &Record) -> anyhow::Result<()> {
        todo!()
    }

    async fn id_to_receipts(&self, user_id: &ARID) -> anyhow::Result<HashSet<Receipt>> {
        todo!()
    }

    async fn receipt_to_record(&self, receipt: &Receipt) -> anyhow::Result<Option<Record>> {
        todo!()
    }

    async fn delete_record(&self, receipt: &Receipt) -> anyhow::Result<()> {
        todo!()
    }

    async fn set_user_key(&self, old_public_key: &PublicKeyBase, new_public_key: &PublicKeyBase) -> anyhow::Result<()> {
        todo!()
    }

    async fn set_user_recovery(&self, user: &User, recovery: Option<&str>) -> anyhow::Result<()> {
        todo!()
    }

    async fn remove_user(&self, user: &User) -> anyhow::Result<()> {
        todo!()
    }

    async fn recovery_to_user(&self, recovery: &str) -> anyhow::Result<Option<User>> {
        todo!()
    }
}

impl Depo {
    pub async fn new_db() -> anyhow::Result<Self> {
        Ok(Self::new(DbDepoImpl::new().await?))
    }
}
