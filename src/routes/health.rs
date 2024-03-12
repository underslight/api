use actix_web::{HttpResponse, Responder};

#[actix_web::get("/health")]
pub(crate) async fn health() -> impl Responder {
    HttpResponse::Ok()
}