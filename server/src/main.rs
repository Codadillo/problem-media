#[macro_use]
extern crate diesel;
mod database;
mod problems;

use actix_cors::Cors;
use actix_session::{CookieSession, Session};
use actix_web::{http, middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use database::models;
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

async fn create_problem(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Json<problems::NewProblem>,
) -> Result<impl Responder, Error> {
    if !validate(&session, pool.clone()).await?
        || req.owner_id != session.get::<models::SessionUser>("user")?.unwrap().id
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
            new_db_problem.insert(&conn)
        },
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    Ok(HttpResponse::Ok().body(new_problem.into_problem()?.id.to_string()))
}

async fn get_problem(
    session: Session,
    pool: web::Data<DbPool>,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if !validate(&session, pool.clone()).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let conn = pool.get().expect("couldn't get db connection from pool");
    let problem = web::block(move || models::DbProblem::get_by_id(id.into_inner(), &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(if let Some(problem) = problem {
        HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&problem).unwrap())
    } else {
        HttpResponse::NotFound().finish()
    })
}

async fn recommend_problem(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Path<(i32, bool)>,
) -> Result<impl Responder, Error> {
    if !validate(&session, pool.clone()).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let conn = pool.get().expect("couldn't get db connection from pool");
    let id = req.0;
    let undo = req.1;
    let user = session.get::<models::SessionUser>("user")?;
    if user.is_none() {
        return Ok(HttpResponse::BadRequest().body("Invalid session"));
    }
    let user: models::NewUser = user.unwrap().into();
    let resp = web::block(move || -> Result<Option<&str>, diesel::result::Error> {
        let user = user.get(&conn)?;
        if let Some(mut user) = user {
            if user.recommended_ids.contains(&id) == !undo {
                return Ok(Some("Attempt to recommend already problem"));
            }
            let problem = models::DbProblem::get_by_id(id, &conn)?;
            if let Some(mut problem) = problem {
                if undo {
                    let id_index = user.recommended_ids.binary_search(&id).unwrap();
                    user.recommended_ids.remove(id_index);
                    problem.recommendations -= 1;
                } else {
                    user.recommended_ids.push(id);
                    problem.recommendations += 1;
                }
                user.update_recommendations(&conn)?;
                problem.update_recommendations(&conn)?;
                Ok(None)
            } else {
                Ok(Some("Could not find requested problem"))
            }
        } else {
            Ok(Some("Could not find session user"))
        }
    }).await.map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    if let Some(error) = resp {
        Ok(HttpResponse::NotFound().body(error))
    } else {
        Ok(HttpResponse::Ok().finish())
    }
}

async fn query_problems(
    session: Session,
    pool: web::Data<DbPool>,
    web::Query(req): web::Query<models::ProblemQuery>,
) -> Result<impl Responder, Error> {
    if !validate(&session, pool.clone()).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let conn = pool.get().expect("couldn't get db connection from pool");
    let problems = web::block(move || req.query(&conn)).await.map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&problems)?))
}

async fn create_user(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Json<models::NewUser>,
) -> Result<impl Responder, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let new_user = web::block(
        move || -> Result<Option<models::User>, diesel::result::Error> {
            if models::User::get_by_name(req.name.clone(), &conn)?.is_some() {
                return Ok(None);
            }
            let req: models::NewUser = req.into_inner().into();
            req.insert(&conn).optional()
        },
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    Ok(if let Some(new_user) = new_user {
        let new_user: models::SessionUser = new_user.into();
        session.set("user", new_user)?;
        HttpResponse::Ok()
    } else {
        HttpResponse::NotAcceptable()
    })
}

async fn login(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Json<models::NewUser>,
) -> Result<impl Responder, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user = web::block(move || req.get(&conn)).await.map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError()
    })?;
    Ok(if let Some(user) = user {
        let user: models::SessionUser = user.into();
        session.set("user", user)?;
        HttpResponse::Ok()
    } else {
        HttpResponse::NotAcceptable()
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
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:8000")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .finish(),
            )
            .service(web::resource("/api/problem").route(web::get().to(query_problems)))
            .service(web::resource("/api/problem/create").route(web::post().to(create_problem)))
            .service(web::resource("/api/problem/{id}").route(web::get().to(get_problem)))
            .service(web::resource("/api/problem/{id}/recommend/{undo}").route(web::get().to(recommend_problem)))
            .service(web::resource("/api/account/login").route(web::post().to(login)))
            .service(web::resource("/api/account/create").route(web::post().to(create_user)))
        // .service(web::resource("*").route(web::get().to(|| HttpResponse::Ok().body("404 page"))))
    })
    .bind(format!("127.0.0.1:{}", PORT))?
    .run()
    .await
}
