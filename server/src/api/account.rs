use crate::validate;
use actix_session::Session;
use actix_web::{http, web, Error, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{database::models, DbPool};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .route("", web::get().to(get_from_session))
            .route("/create", web::get().to(create))
            .route("/login", web::post().to(login))
            .route("/{id}", web::get().to(get)),
    );
}

async fn get_from_session(
    session: Session,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, Error> {
    let session_user = session.get::<models::SessionUser>("user")?;
    if let Some(session_user) = session_user {
        get(session, pool, session_user.id.into()).await
    } else {
        Ok(HttpResponse::BadRequest().finish())
    }
}

async fn get(
    session: Session,
    pool: web::Data<DbPool>,
    req: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    if !validate(&session, pool.clone()).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let id = req.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");
    let resp = web::block(move || models::User::get_by_id(id, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(if let Some(user) = resp {
        HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&user)?)
    } else {
        HttpResponse::NotFound().body("Could not find user")
    })
}

async fn create(
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
