use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    username: String,
    password: String,
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
