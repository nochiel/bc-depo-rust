use bc_components::PublicKeyBase;

#[derive(Clone, Debug)]
pub struct RecoveryContinuation {
    pub old_key: PublicKeyBase,
    pub new_key: PublicKeyBase,
    pub expiry: dcbor::Date,
}

impl RecoveryContinuation {
    pub fn new(old_key: PublicKeyBase, new_key: PublicKeyBase, expiry: dcbor::Date) -> Self {
        Self {
            old_key,
            new_key,
            expiry,
        }
    }

    pub fn old_key(&self) -> &PublicKeyBase {
        &self.old_key
    }

    pub fn new_key(&self) -> &PublicKeyBase {
        &self.new_key
    }

    pub fn expiry(&self) -> &dcbor::Date {
        &self.expiry
    }
}
