use bc_components::{PublicKeyBase, ARID};

#[derive(Debug)]
pub struct User {
    user_id: ARID,
    public_key: PublicKeyBase,
    fallback: Option<String>,
}

impl User {
    pub fn new(user_id: ARID, public_key: PublicKeyBase, fallback: Option<String>) -> Self {
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

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
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
        let user = User::new(ARID::new(), public_key, None);
        println!("{:?}", user);
    }
}
