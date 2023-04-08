extern crate bcrypt;

use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> String {
    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    hashed_password
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    let is_password_correct = verify(password, hashed_password).unwrap();
    is_password_correct
}
