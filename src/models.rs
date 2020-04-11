use serde::{Deserialize, Serialize};

use crate::problems::Problem;
use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
    pub pass: String,
}

impl From<User> for NewUser {
    fn from(user: User) -> NewUser {
        NewUser {
            name: user.name,
            pass: user.pass,
        }
    }
}

#[derive(Debug, Clone, Serialize, Queryable)]
pub struct DbProblem {
    pub id: i32,
    pub owner_id: i32,
    pub p_type: String,
    pub topic: String,
    pub tags: Vec<String>,
    pub data: String,
}

impl DbProblem {
    pub fn into_problem(self) -> Result<Problem, serde_json::Error> {
        Ok(Problem {
            id: self.id,
            owner_id: self.owner_id,
            topic: serde_json::from_str(&self.topic)?,
            tags: self.tags,
            content: serde_json::from_str(&self.data)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "problems"]
pub struct NewDbProblem {
    pub owner_id: i32,
    pub p_type: String,
    pub topic: String,
    pub tags: Vec<String>,
    pub data: String,
}
