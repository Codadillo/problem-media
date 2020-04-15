use actix_session::Session;
use actix_web::{web, Error, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{database::models, DbPool};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .route("/create", web::get().to(create))
            .route("/login", web::post().to(login)),
    );
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
