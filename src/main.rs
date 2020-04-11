#[macro_use]
extern crate diesel;
mod database;
mod problems;

use actix_session::{CookieSession, Session};
use actix_web::{middleware, post, web, App, Error, HttpResponse, HttpServer, Responder};
use database::{actions, models};
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use env_logger::Env;

const PORT: i32 = 8080;
const POSTGRES_CON: &'static str =
    "host=localhost port=5432 user=postgres password=postgres dbname=akshardb";

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn validate(session: &Session, pool: web::Data<DbPool>) -> Result<bool, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user = session.get::<models::User>("user")?;
    if let Some(user) = user {
        let valid = web::block(move || -> Result<bool, diesel::result::Error> {
            Ok(actions::get_user(user.into(), &conn)?.is_some())
        })
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
        Ok(valid)
    } else {
        Ok(false)
    }
}

async fn create_problem(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Json<problems::NewProblem>,
) -> Result<impl Responder, Error> {
    if !validate(&session, pool.clone()).await?
        || req.owner_id != session.get::<models::User>("user")?.unwrap().id
    {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let conn = pool.get().expect("couldn't get db connection from pool");
    let new_db_problem = req
        .into_inner()
        .into_new_db_problem()
        .map_err(|_| HttpResponse::BadRequest().finish())?;
    let new_problem = web::block(
        move || -> Result<models::DbProblem, diesel::result::Error> {
            actions::insert_problem(new_db_problem, &conn)
        },
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    Ok(HttpResponse::Ok().body(new_problem.into_problem()?.id.to_string()))
}

#[post("/api/account/create")]
async fn create_user(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Form<models::NewUser>,
) -> Result<impl Responder, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let new_user = web::block(
        move || -> Result<Option<models::User>, diesel::result::Error> {
            if actions::get_user_by_name(req.name.clone(), &conn)?.is_some() {
                return Ok(None);
            }
            actions::insert_user(req.into_inner(), &conn).optional()
        },
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    if let Some(new_user) = new_user {
        session.set("user", new_user)?;
        Ok(HttpResponse::Ok())
    } else {
        Ok(HttpResponse::BadRequest())
    }
}

#[post("/api/account/login")]
async fn login(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Form<models::NewUser>,
) -> Result<impl Responder, Error> {
    session.set("user", req.into_inner())?;
    Ok(if validate(&session, pool).await? {
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    })
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Setup DB stuff
    let manager = ConnectionManager::<PgConnection>::new(POSTGRES_CON);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    // Set up logger
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    // Run the server
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                CookieSession::signed(&[0; 32])
                    .name("actix-session")
                    .path("/")
                    .secure(false),
            )
            .service(create_user)
            .service(web::resource("/api/problem/create").route(web::post().to(create_problem)))
    })
    .bind(format!("127.0.0.1:{}", PORT))?
    .run()
    .await
}
