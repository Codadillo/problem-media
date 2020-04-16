#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;

mod api;
mod database;

use std::env;

use actix_cors::Cors;
use actix_session::{CookieSession, Session};
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer};
use database::models;
use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager},
};
use env_logger::Env;

fn get_env_with_dev_default(key: &'static str, default: &'static str) -> String {
    env::var(key)
        .map_err(|_| {
            if cfg!(debug_assertions) {
                Some(default)
            } else {
                None
            }
        })
        .expect(format!("Expected {} to be set", key).as_str())
}

lazy_static! {
    static ref APP_HOST: String = get_env_with_dev_default("APP_HOST", "localhost");
    static ref APP_PORT: String = get_env_with_dev_default("APP_PORT", "8080");
    static ref DB_HOST: String = get_env_with_dev_default("DB_HOST_URL", "localhost");
    static ref DB_PORT: String = get_env_with_dev_default("DB_PORT", "5432");
    static ref DB_USER: String = get_env_with_dev_default("DB_USER", "postgres");
    static ref DB_PASSWORD: String = get_env_with_dev_default("DB_PASSWORD", "postgres");
    static ref DB_NAME: String = get_env_with_dev_default("DB_NAME", "akshardb");

    static ref APP_ADDR: String = format!("{}:{}", *APP_HOST, *APP_PORT);
    static ref DB_URL: String = format!("host={} port={} user={} password={} dbname={}", *DB_HOST, *DB_PORT, *DB_USER, *DB_PASSWORD, *DB_NAME);
}

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
    let manager = ConnectionManager::<PgConnection>::new(DB_URL.as_str());
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
                    .domain(APP_HOST.to_string())
                    .path("/")
                    .secure(false),
            )
            .wrap(
                // TODO: Use separate CORS headers for debug/release
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
    .bind(APP_ADDR.clone())?
    .run()
    .await
}
