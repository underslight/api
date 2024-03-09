use actix_web::{web, Scope};

pub mod authenticate;
pub mod add;
pub mod remove;
pub mod update;
pub mod refresh;
pub mod delete;
pub mod register;

pub(crate) fn scope() -> Scope {
    web::scope("/auth")
        .service(authenticate::scope())
        .service(register::scope())
        .service(remove::scope())
        .service(add::scope())
        .service(delete::delete)
        .service(refresh::refresh)
}