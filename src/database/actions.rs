use super::models::*;
use super::schema;
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

pub fn insert_problem(
    new_problem: NewDbProblem,
    conn: &PgConnection,
) -> Result<DbProblem, result::Error> {
    use schema::problems::dsl::*;
    diesel::insert_into(problems)
        .values(&new_problem)
        .get_result(conn)
}
