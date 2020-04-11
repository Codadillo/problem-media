use super::{models::*, schema};
use crate::problems;
use diesel::{prelude::*, result};

pub fn get_user(user: NewUser, conn: &PgConnection) -> Result<Option<User>, result::Error> {
    use schema::users::dsl::*;
    users
        .filter(name.eq(user.name))
        .filter(pass.eq(user.pass))
        .get_result(conn)
        .optional()
}

pub fn get_user_by_name(
    user_name: String,
    conn: &PgConnection,
) -> Result<Option<User>, result::Error> {
    use schema::users::dsl::*;
    users.filter(name.eq(user_name)).get_result(conn).optional()
}

pub fn insert_user(new_user: NewUser, conn: &PgConnection) -> Result<User, result::Error> {
    use schema::users::dsl::*;
    diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)
}

pub fn query_problems(
    req: problems::ProblemQuery,
    conn: &PgConnection,
) -> Result<Vec<i32>, result::Error> {
    use schema::problems::dsl::*;
    let mut query = problems.select(id).into_boxed();
    if let Some(p_id) = req.id {
        query = query.filter(id.eq(p_id));
    }
    if let Some(p_owner_id) = req.owner_id {
        query = query.filter(owner_id.eq(p_owner_id));
    }
    if let Some(p_topic) = req.topic {
        query = query.filter(topic.eq(serde_json::to_string(&p_topic).unwrap()));
    }
    if let Some(p_tags) = req.tags {
        query = query.filter(tags.eq(p_tags));
    }
    if let Some(pr_type) = req.problem_type {
        query = query.filter(p_type.eq(serde_json::to_string(&pr_type).unwrap()));
    }
    query
        .limit(req.max_results.unwrap_or(50) as i64)
        .get_results(conn)
}

pub fn insert_problem(
    new_problem: NewDbProblem,
    conn: &PgConnection,
) -> Result<DbProblem, result::Error> {
    use schema::problems::dsl::*;
    diesel::insert_into(problems)
        .values(&new_problem)
        .get_result(conn)
}
