use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UserRaw {
    pub name: String,
    pub email: String,
}
