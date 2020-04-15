mod account;
mod problems;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .service(web::scope("/account").configure(account::config))
            .service(web::scope("/problems").configure(problems::config)),
    );
}
