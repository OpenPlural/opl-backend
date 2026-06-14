use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{Error, SaltString};
use argon2::password_hash::rand_core::OsRng;
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use rand::{rng, Rng};
use rand::distr::Alphanumeric;
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

static PASSWORD_HASH_ALGORITHM: OnceCell<Argon2> = OnceCell::const_new();

pub const SESSION_TOKEN_LENGTH: usize = 128;
pub const API_KEY_TOKEN_LENGTH: usize = 126;
pub const API_KEY_TOKEN_PREFIX: &str = "k-";
const SHA256_PEPPER: &'static str = "OpenPlural";

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

pub fn sha256(input: &str) -> String {
    let full_input = format!("{}{}{}", SHA256_PEPPER, input, SHA256_PEPPER);
    let hash = Sha256::digest(full_input.as_bytes());
    BASE64_STANDARD_NO_PAD.encode(hash)
}