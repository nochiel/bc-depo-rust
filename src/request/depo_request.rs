use bc_envelope::prelude::*;
use bc_components::PublicKeyBase;

pub trait DepoRequest: RequestBody {
    fn public_key(&self) -> &PublicKeyBase;
}
