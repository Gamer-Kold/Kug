use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn default() -> Self {
        Self { 
            username: String::from("???"),
            password: String::from("???"),
        }
    }
}
