use std::collections::HashSet;

use async_trait::async_trait;
use bc_components::PublicKeyBase;
use bytes::Bytes;

use crate::receipt::Receipt;

#[async_trait]
pub trait Store {
    /// This is a Trust-On-First-Use (TOFU) function. If the provided public key is not
    /// recognized, then a new account is created and the provided payload is stored in
    /// it. It is also used to add additional shares to an existing account. Adding an
    /// already existing share to an account is idempotent.
    async fn store(&self, public_key: &PublicKeyBase, payload: Bytes) -> anyhow::Result<Receipt>;

    /// Updates an account's fallback contact method, which could be a phone
    /// number, email address, or similar. The fallback is used to give users a
    /// way to change their public key in the event they lose it. It is up to
    /// the implementer to validate the fallback contact method before letting
    /// the public key be changed. If the fallback is `None`, then the fallback
    /// contact method is deleted.
    async fn update_fallback(&self, public_key: &PublicKeyBase, fallback: Option<&str>) -> anyhow::Result<()>;

    /// Retrieves an account's fallback contact method, if any.
    async fn get_fallback(&self, public_key: &PublicKeyBase) -> anyhow::Result<Option<String>>;

    /// Changes the public key used as the account identifier. It could be invoked
    /// specifically because a user requests it, in which case they will need to know
    /// their old public key, or it could be invoked because they used their fallback
    /// contact method to request a transfer token that encodes their old public key.
    async fn change_public_key(&self, old_public_key: &PublicKeyBase, new_public_key: &PublicKeyBase) -> anyhow::Result<()>;

    /// Deletes either a subset of shares a user controls, or all the shares if a
    /// subset of receipts is not provided. Deletes are idempotent; in other words,
    /// deleting nonexistent shares is not an error.
    async fn delete(&self, public_key: &PublicKeyBase, receipts: &HashSet<Receipt>) -> anyhow::Result<()>;

    /// Returns a dictionary of `[Receipt: Payload]` corresponding to the set of
    /// input receipts, or corresponding to all the controlled shares if no input
    /// receipts are provided. Attempting to retrieve nonexistent receipts or receipts
    /// from the wrong account is an error.
    async fn retrieve(&self, public_key: &PublicKeyBase, receipts: &HashSet<Receipt>) -> anyhow::Result<HashSet<(Receipt, Bytes)>>;

    /// Deletes all the shares of an account and any other data associated with it, such
    /// as the fallback contact method. Deleting an account is idempotent; in other words,
    /// deleting a nonexistent account is not an error.
    async fn delete_account(&self, public_key: &PublicKeyBase) -> anyhow::Result<()>;

    /// Requests a reset of the account's public key without knowing the current one.
    /// The account must have a validated fallback contact method that matches the one
    /// provided. The Store owner needs to then contact the user via their fallback
    /// contact method to confirm the change. If the request is not confirmed by a set
    /// amount of time, then the change is not made.
    async fn request_reset(&self, fallback: &str, new_public_key: &PublicKeyBase) -> anyhow::Result<()>;
}
