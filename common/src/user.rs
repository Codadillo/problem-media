use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub pass: String,
    pub recommended_ids: Vec<i32>,
}