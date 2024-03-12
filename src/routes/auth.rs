use actix_web::{web, Scope};

pub mod authenticate;
pub mod link;
pub mod unlink;
pub mod update;
pub mod refresh;
pub mod delete;
pub mod register;

pub(crate) fn scope() -> Scope {
    web::scope("/auth")
        .service(authenticate::scope())
        .service(register::scope())
        .service(unlink::scope())
        .service(link::scope())
        .service(delete::delete)
        .service(refresh::refresh)
        .service(update::update)
}