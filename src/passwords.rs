use bcrypt::{hash, verify, DEFAULT_COST};
use std::result;

#[derive(Debug)]
pub enum PasswordError {
    HashError,
    VerifyError,
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PasswordError::HashError => write!(f, "Error hashing password"),
            PasswordError::VerifyError => write!(f, "Error verifying password"),
        }
    }
}

impl std::error::Error for PasswordError {}

type Result<T> = result::Result<T, PasswordError>;

pub fn hash_password(password: &str) -> Result<String> {
    hash(password, DEFAULT_COST).map_err(|_| PasswordError::HashError)
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool> {
    verify(password, hashed_password).map_err(|_| PasswordError::VerifyError)
}
