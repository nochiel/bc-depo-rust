use bc_components::{PublicKeyBase, ARID};

#[derive(Debug, Clone)]
pub struct User {
    user_id: ARID,
    public_key: PublicKeyBase,
    fallback: Option<String>,
}

impl User {
    pub fn new(user_id: ARID, public_key: PublicKeyBase) -> Self {
        Self::new_with_fallback(user_id, public_key, None)
    }

    pub fn new_with_fallback(user_id: ARID, public_key: PublicKeyBase, fallback: Option<String>) -> Self {
        Self {
            user_id,
            public_key,
            fallback,
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

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }

    pub fn set_fallback(&mut self, fallback: Option<&str>) {
        self.fallback = fallback.map(str::to_owned);
    }
}

#[cfg(test)]
mod tests {
    use bc_components::PrivateKeyBase;

    use super::*;

    #[test]
    fn test_1() {
        let private_key = PrivateKeyBase::new();
        let public_key = private_key.public_keys();
        let user = User::new(ARID::new(), public_key);
        println!("{:?}", user);
    }
}
