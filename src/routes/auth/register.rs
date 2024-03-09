use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::email_password::EmailPasswordMethod, token::IdToken, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;
use auth::builder::*;

#[derive(Debug, Deserialize)]
pub struct EmailPasswordRequest {
    pub email: String,
    pub password: String,
    pub return_id_token: bool,
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub id_token: Option<IdToken>,
    pub user: User,
}

#[actix_web::post("/EmailPassword")]
pub(crate) async fn email_password(Json(request): Json<EmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Creates the credential
    let credential = EmailPasswordMethod::new(request.email, request.password);

    // Creates the user
    let user = User::builder()
        .build_safe()
        .save(&db, Box::new(credential))
        .await?;

    // Generates an ID token
    let id_token = match request.return_id_token {
        true => Some(user.get_id_token()?),
        false => None,
    };

    Ok(Json(
        RegistrationResponse {
            id_token,
            user,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/register")
        .service(email_password)
}