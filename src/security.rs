use argon2::Argon2;
use rand::{rng, Rng};
use rand::distr::Alphanumeric;
use tokio::sync::OnceCell;

static PASSWORD_HASH_ALGORITHM: OnceCell<Argon2> = OnceCell::const_new();

pub const SESSION_TOKEN_LENGTH: usize = 255;

pub async fn get_password_hash_algorithm() -> &'static Argon2<'static> {
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