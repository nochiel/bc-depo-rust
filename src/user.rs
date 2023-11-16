use bc_components::{PublicKeyBase, ARID};

#[derive(Debug, Clone)]
pub struct User {
    user_id: ARID,
    public_key: PublicKeyBase,
    recovery: Option<String>,
}

impl User {
    pub fn new(user_id: ARID, public_key: PublicKeyBase) -> Self {
        Self::new_opt(user_id, public_key, None)
    }

    pub fn new_opt(user_id: ARID, public_key: PublicKeyBase, recovery: Option<String>) -> Self {
        Self {
            user_id,
            public_key,
            recovery,
        }
    }

    pub fn user_id(&self) -> &ARID {
        &self.user_id
    }

    pub fn public_key(&self) -> &PublicKeyBase {
        &self.public_key
    }

    pub fn set_public_key(&mut self, public_key: PublicKeyBase) {
        self.public_key = public_key;
    }

    pub fn recovery(&self) -> Option<&str> {
        self.recovery.as_deref()
    }

    pub fn set_recovery(&mut self, recovery: Option<&str>) {
        self.recovery = recovery.map(str::to_owned);
    }
}
