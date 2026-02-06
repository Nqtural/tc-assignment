use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct NewUser {
    pub name: String,
    pub surname: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub surname: String,
    pub password_hash: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct NewRoom {
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct Room {
    pub id: i32,
    pub name: String,
    pub description: String,
}
