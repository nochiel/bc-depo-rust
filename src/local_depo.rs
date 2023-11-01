use std::collections::{HashSet, HashMap};

use anyhow::bail;
use async_trait::async_trait;
use bc_components::{PublicKeyBase, ARID, PrivateKeyBase};
use bytes::Bytes;

use crate::{receipt::Receipt, user::User, record::Record, recovery_continuation::RecoveryContinuation};

#[async_trait]
pub trait LocalDepo {
    fn max_payload_size(&self) -> usize;
    fn continuation_expiry_seconds(&self) -> f64;
    fn private_key(&self) -> &PrivateKeyBase;
    fn public_key(&self) -> &PublicKeyBase;
    async fn existing_key_to_id(&self, key: &PublicKeyBase) -> anyhow::Result<Option<ARID>>;
    async fn existing_id_to_user(&self, user_id: &ARID) -> anyhow::Result<Option<User>>;
    async fn insert_user(&self, user: &User) -> anyhow::Result<()>;
    async fn insert_record(&self, record: &Record) -> anyhow::Result<()>;
    async fn id_to_receipts(&self, user_id: &ARID) -> anyhow::Result<HashSet<Receipt>>;
    async fn receipt_to_record(&self, receipt: &Receipt) -> anyhow::Result<Record>;
    async fn delete_record(&self, receipt: &Receipt) -> anyhow::Result<()>;
    async fn set_user_key(&self, old_key: &PublicKeyBase, new_key: &PublicKeyBase) -> anyhow::Result<()>;
    async fn set_user_recovery(&self, user: &User, recovery: Option<&str>) -> anyhow::Result<()>;
    async fn remove_user(&self, user: &User) -> anyhow::Result<()>;
    async fn recovery_to_user(&self, recovery: &str) -> anyhow::Result<Option<User>>;

    async fn records_for_id_and_receipts(&self, user_id: &ARID, recipts: &HashSet<Receipt>) -> anyhow::Result<Vec<Record>> {
        let mut result = Vec::new();
        let user_receipts = self.id_to_receipts(user_id).await?;
        for receipt in recipts {
            if !user_receipts.contains(receipt) {
                bail!("unknown receipt");
            }
            let record = self.receipt_to_record(receipt).await?;
            result.push(record.clone());
        }
        Ok(result)
    }

    async fn existing_key_to_user(&self, key: &PublicKeyBase) -> anyhow::Result<Option<User>> {
        let user_id = self.existing_key_to_id(key).await?;
        let user = match user_id {
            Some(user_id) => self.existing_id_to_user(&user_id).await?,
            None => return Ok(None),
        };
        Ok(user)
    }

    async fn key_to_user(&self, key: &PublicKeyBase) -> anyhow::Result<User> {
        let user = self.existing_key_to_user(key).await?;
        let user = match user {
            Some(user_id) => user_id,
            None => {
                let user_id = ARID::new();
                let user = User::new(user_id.clone(), key.clone());
                self.insert_user(&user).await?;
                user
            }
        };
        Ok(user)
    }

    async fn expect_key_to_user(&self, key: &PublicKeyBase) -> anyhow::Result<User> {
        let user_id = self.existing_key_to_user(key).await?;
        let user_id = match user_id {
            Some(user_id) => user_id,
            None => bail!("unknown public key"),
        };
        Ok(user_id)
    }

    /// This is a Trust-On-First-Use (TOFU) function. If the provided public key is not
    /// recognized, then a new account is created and the provided data is stored in
    /// it. It is also used to add additional shares to an existing account. Adding an
    /// already existing share to an account is idempotent.
    async fn store_share(&self, key: &PublicKeyBase, data: &Bytes) -> anyhow::Result<Receipt> {
        let user = self.key_to_user(key).await?;
        if data.len() > self.max_payload_size() {
            bail!("data too large");
        }
        let record = Record::new(user.user_id(), data);
        self.insert_record(&record).await?;
        Ok(record.receipt().clone())
    }

    /// Returns a dictionary of `[Receipt: Payload]` corresponding to the set of
    /// input receipts, or corresponding to all the controlled shares if no input
    /// receipts are provided. Attempting to retrieve nonexistent receipts or receipts
    /// from the wrong account is an error.
    async fn get_shares(&self, key: &PublicKeyBase, receipts: &HashSet<Receipt>) -> anyhow::Result<HashMap<Receipt, Bytes>> {
        let user = self.expect_key_to_user(key).await?;
        let receipts = if receipts.is_empty() {
            self.id_to_receipts(user.user_id()).await?
        } else {
            receipts.clone()
        };
        let records = self.records_for_id_and_receipts(user.user_id(), &receipts).await?;
        let mut result = HashMap::new();
        for record in records {
            result.insert(record.receipt().clone(), record.data().clone());
        }
        Ok(result)
    }

    /// Deletes either a subset of shares a user controls, or all the shares if a
    /// subset of receipts is not provided. Deletes are idempotent; in other words,
    /// deleting nonexistent shares is not an error.
    async fn delete_shares(&self, key: &PublicKeyBase, receipts: &HashSet<Receipt>) -> anyhow::Result<()> {
        let user = self.expect_key_to_user(key).await?;
        let recpts = if receipts.is_empty() {
            self.id_to_receipts(user.user_id()).await?
        } else {
            receipts.clone()
        };
        for receipt in recpts {
            self.delete_record(&receipt).await?;
        }
        Ok(())
    }

    /// Changes the public key used as the account identifier. It could be invoked
    /// specifically because a user requests it, in which case they will need to know
    /// their old public key, or it could be invoked because they used their recovery
    /// contact method to request a transfer token that encodes their old public key.
    async fn update_key(&self, old_key: &PublicKeyBase, new_key: &PublicKeyBase) -> anyhow::Result<()> {
        if self.existing_key_to_id(new_key).await?.is_some() {
            bail!("public key already in use");
        }
        self.set_user_key(old_key, new_key).await?;
        Ok(())
    }

    /// Deletes all the shares of an account and any other data associated with it, such
    /// as the recovery contact method. Deleting an account is idempotent; in other words,
    /// deleting a nonexistent account is not an error.
    async fn delete_account(&self, key: &PublicKeyBase) -> anyhow::Result<()> {
        let user = self.expect_key_to_user(key).await?;
        self.delete_shares(key, &HashSet::new()).await?;
        self.remove_user(&user).await?;
        Ok(())
    }

    /// Updates an account's recovery contact method, which could be a phone
    /// number, email address, or similar. The recovery is used to give users a
    /// way to change their public key in the event they lose it. It is up to
    /// the implementer to validate the recovery contact method before letting
    /// the public key be changed. If the recovery is `None`, then the recovery
    /// contact method is deleted.
    async fn update_recovery(&self, key: &PublicKeyBase, recovery: Option<&str>) -> anyhow::Result<()> {
        let user = self.expect_key_to_user(key).await?;
        // Recoverys must be unique
        if let Some(non_opt_recovery) = recovery {
            let existing_recovery_user = self.recovery_to_user(non_opt_recovery).await?;
            if let Some(existing_recovery_user) = existing_recovery_user {
                if existing_recovery_user.user_id() != user.user_id() {
                    bail!("recovery already in use");
                } else {
                    // The user is already using this recovery, so we can just return
                    // (idempotency)
                    return Ok(());
                }
            }
        }
        self.set_user_recovery(&user, recovery).await?;
        Ok(())
    }

    /// Retrieves an account's recovery contact method, if any.
    async fn get_recovery(&self, key: &PublicKeyBase) -> anyhow::Result<Option<String>> {
        let user = self.expect_key_to_user(key).await?;
        let recovery = user.recovery().map(|s| s.to_string());
        Ok(recovery)
    }

    /// Requests a reset of the account's public key without knowing the current
    /// one. The account must have a validated recovery contact method that
    /// matches the one provided. The depository owner needs to then contact the
    /// user via their recovery contact method to confirm the change. If the
    /// request is not confirmed by a set amount of time, then the change is not
    /// made.
    ///
    /// Recoverys must be unique. Examples of possible recoverys some sort of
    /// username, real name, or other unique identifier, paired with an email
    /// addresses, phone number, list of security questions, two-factor
    /// authentication key for time-based one-time passwords, list of trusted
    /// devices for 2FA, or similar.
    ///
    /// Returns a continuation, which is a token that can be used to complete
    /// the reset.
    async fn start_recovery_transfer(&self, recovery: &str, new_key: &PublicKeyBase) -> anyhow::Result<RecoveryContinuation> {
        // First find the user for the recovery.
        let user = self.recovery_to_user(recovery).await?;
        // If no recovery was found return an error.
        let user = match user {
            Some(user) => user,
            None => bail!("unknown recovery"),
        };
        // Ensure there is no account with the new public key
        let existing_user = self.existing_key_to_id(new_key).await?;
        if existing_user.is_some() {
            bail!("public key already in use");
        }
        Ok(RecoveryContinuation::new(
            user.public_key().clone(),
            new_key.clone(),
            dcbor::Date::now() + self.continuation_expiry_seconds()
        ))
    }

    /// Completes a reset of the account's public key. This is called after the
    /// user has confirmed the change via their recovery contact method.
    async fn finish_recovery_transfer(&self, continuation: &RecoveryContinuation) -> anyhow::Result<()> {
        // Ensure the continuation is valid
        let seconds_until_expiry = continuation.expiry().clone() - dcbor::Date::now();
        if seconds_until_expiry < 0.0 {
            bail!("continuation expired");
        }
        // Set the user's public key to the new public key
        self.set_user_key(continuation.old_key(), continuation.new_key()).await?;
        Ok(())
    }
}
