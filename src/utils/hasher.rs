use bcrypt::hash;

const BCRYPT_COST: u32 = 10;

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, BCRYPT_COST).map_err(|_| "failed to hash password".to_string())
}
