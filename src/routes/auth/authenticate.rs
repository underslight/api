use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::{email_password::EmailPasswordMethod, AuthMethod, MfaCode}, token::IdToken, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct EmailPasswordRequest {
    pub email: String,
    pub password: String,
    pub mfa_code: Option<MfaCode>,
    pub return_id_token: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthenticationResponse {
    pub id_token: Option<IdToken>,
    pub user: User,
}

#[actix_web::post("/EmailPassword")]
pub(crate) async fn email_password(Json(request): Json<EmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {
    
    // Creates the credential
    let credential = EmailPasswordMethod::new(request.email, request.password);

    // Fetches the user
    let user = credential
        .authenticate(&db, request.mfa_code)
        .await?;

    // Generates an ID token
    let id_token = match request.return_id_token {
        true => Some(user.get_id_token()?),
        false => None,
    };

    Ok(Json(
        AuthenticationResponse {
            id_token,
            user,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/authenticate")
        .service(email_password)
}