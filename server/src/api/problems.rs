use actix_session::Session;
use actix_web::{http, web, Error, HttpResponse, Responder};
use common::problems;

use crate::{database::models, validate, DbPool};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .route("", web::get().to(index))
            .route("", web::post().to(create))
            .service(
                web::scope("{id}")
                    .route("/", web::get().to(get))
                    .route("/recommend/{undo}", web::get().to(recommend)),
            ),
    );
}

async fn index(
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

async fn create(
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
    let new_db_problem = models::NewDbProblem::from_new_problem(req.into_inner())
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

async fn get(
    session: Session,
    pool: web::Data<DbPool>,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    if !validate(&session, pool.clone()).await? {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let conn = pool.get().expect("couldn't get db connection from pool");
    let db_problem = web::block(move || models::DbProblem::get_by_id(id.into_inner(), &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(if let Some(db_problem) = db_problem {
        let problem: problems::Problem = db_problem.into_problem()?;
        HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&problem).unwrap())
    } else {
        HttpResponse::NotFound().finish()
    })
}

// TODO: Refactor to handle new routing
async fn recommend(
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
    let resp = web::block(
        move || -> Result<Result<i32, &str>, diesel::result::Error> {
            let user = user.get(&conn)?;
            if let Some(mut user) = user {
                if user.recommended_ids.contains(&id) == !undo {
                    return Ok(Err("Attempt to recommend already recommended problem"));
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
                    Ok(Ok(problem.recommendations))
                } else {
                    Ok(Err("Could not find requested problem"))
                }
            } else {
                Ok(Err("Could not find session user"))
            }
        },
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    Ok(match resp {
        Ok(rec_count) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(rec_count.to_string()),
        Err(error) => HttpResponse::NotFound().body(error),
    })
}
