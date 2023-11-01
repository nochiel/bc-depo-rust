use bc_envelope::prelude::*;
use bc_components::PublicKeyBase;

pub mod store_share;
pub mod delete_shares;
pub mod delete_account;
pub mod get_shares;
pub mod update_key;
pub mod update_fallback;

pub trait DepoRequest: RequestBody {
    fn key(&self) -> &PublicKeyBase;

    fn key_param() -> Parameter {
        Parameter::new_named("key")
    }
}
