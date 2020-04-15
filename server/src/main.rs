#[macro_use]
extern crate diesel;

mod api;
mod database;

use actix_cors::Cors;
use actix_session::{CookieSession, Session};
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer};
use database::models;
use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager},
};
use env_logger::Env;

const PORT: i32 = 8080;
const POSTGRES_CON: &'static str =
    "host=localhost port=5432 user=postgres password=postgres dbname=akshardb";

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// TODO: Convert this into middleware.
pub async fn validate(session: &Session, pool: web::Data<DbPool>) -> Result<bool, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user = session.get::<models::SessionUser>("user")?;
    if let Some(user) = user {
        let valid = web::block(move || -> Result<bool, diesel::result::Error> {
            let user: models::NewUser = user.into();
            Ok(user.get(&conn)?.is_some())
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
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:8000")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .finish(),
            )
            .service(web::scope("/api").configure(api::config))
            .service(
                web::resource("*").route(web::get().to(|| HttpResponse::Ok().body("404 page"))),
            )
    })
    .bind(format!("127.0.0.1:{}", PORT))?
    .run()
    .await
}
