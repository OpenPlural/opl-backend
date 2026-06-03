use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{Error, SaltString};
use argon2::password_hash::rand_core::OsRng;
use rand::{rng, Rng};
use rand::distr::Alphanumeric;
use tokio::sync::OnceCell;

static PASSWORD_HASH_ALGORITHM: OnceCell<Argon2> = OnceCell::const_new();

pub const SESSION_TOKEN_LENGTH: usize = 255;

async fn get_hash_algorithm() -> &'static Argon2<'static> {
    PASSWORD_HASH_ALGORITHM.get_or_init(|| async {
        Argon2::default()
    }).await
}

pub fn random_string(length: usize) -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub async fn hash(input: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = get_hash_algorithm().await.hash_password(input.as_bytes(), &salt)?;
    let hash = hash.to_string();
    Ok(hash)
}

pub async fn verify(hash: &str, input: &str) -> Result<(), Error> {
    let hash = PasswordHash::new(hash)?;
    get_hash_algorithm().await.verify_password(input.as_bytes(), &hash)?;
    Ok(())
}