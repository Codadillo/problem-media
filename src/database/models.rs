use super::schema::{self, *};
use crate::problems::{Problem, Topic, ProblemType};
use serde::{Deserialize, Serialize};
use diesel::{prelude::*, result};

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

impl NewUser {
    pub fn get_by_name(
        user_name: String,
        conn: &PgConnection,
    ) -> Result<Option<User>, result::Error> {
        use schema::users::dsl::*;
        users.filter(name.eq(user_name)).get_result(conn).optional()
    }

    pub fn get(&self, conn: &PgConnection) -> Result<Option<User>, result::Error> {
        use schema::users::dsl::*;
        users
            .filter(name.eq(self.name.clone()))
            .filter(pass.eq(self.pass.clone()))
            .get_result(conn)
            .optional()
    }

    pub fn insert(&self, conn: &PgConnection) -> Result<User, result::Error> {
        use schema::users::dsl::*;
        diesel::insert_into(users)
            .values(self)
            .get_result(conn)
    }
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

impl NewDbProblem {
    pub fn insert(
        &self,
        conn: &PgConnection,
    ) -> Result<DbProblem, result::Error> {
        use schema::problems::dsl::*;
        diesel::insert_into(problems)
            .values(self)
            .get_result(conn)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemQuery {
    #[serde(default)]
    pub id: Option<i32>,
    #[serde(default)]
    pub owner_id: Option<i32>,
    #[serde(default)]
    pub topic: Option<Topic>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub problem_type: Option<ProblemType>,
    #[serde(default)]
    pub max_results: Option<usize>,
}

impl ProblemQuery {
    pub fn query(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<i32>, result::Error> {
        use schema::problems::dsl::*;
        let mut query = problems.select(id).into_boxed();
        if let Some(p_id) = self.id {
            query = query.filter(id.eq(p_id));
        }
        if let Some(p_owner_id) = self.owner_id {
            query = query.filter(owner_id.eq(p_owner_id));
        }
        if let Some(p_topic) = &self.topic {
            query = query.filter(topic.eq(serde_json::to_string(p_topic).unwrap()));
        }
        if let Some(p_tags) = &self.tags {
            query = query.filter(tags.eq(p_tags));
        }
        if let Some(pr_type) = &self.problem_type {
            query = query.filter(p_type.eq(serde_json::to_string(pr_type).unwrap()));
        }
        query
            .limit(self.max_results.unwrap_or(50) as i64)
            .get_results(conn)
    }
}
