use actix_session::Session;
use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::email_password::EmailPasswordMethod, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;
use auth::builder::*;

#[derive(Debug, Deserialize)]
pub struct EmailPasswordRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub user: User,
}

#[actix_web::post("/EmailPassword")]
pub(crate) async fn email_password(session: Session, Json(request): Json<EmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Creates the credential
    let credential = EmailPasswordMethod::new(request.email, request.password);

    // Creates the user
    let user = User::builder()
        .build_safe()
        .save(&db, Box::new(credential))
        .await?;

    // Generates an ID token and saves it as an httpOnly cookie
    let id_token = user.get_id_token()?;
    session.insert("access_token", id_token.access)?;
    session.insert("refresh_token", id_token.refresh)?;

    Ok(Json(
        RegistrationResponse {
            user,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/register")
        .service(email_password)
}