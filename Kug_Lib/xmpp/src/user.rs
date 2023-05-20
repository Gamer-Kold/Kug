use std::str::FromStr;

use jid::Jid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: Jid,
    pub password: String,
}

impl User {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: Jid::from_str(username).unwrap(),
            password: String::from(password)
        }
    }
}
