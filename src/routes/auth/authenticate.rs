use actix_session::Session;
use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::{email_password::EmailPasswordMethod, AuthMethod, MfaCode}, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct EmailPasswordRequest {
    pub email: String,
    pub password: String,
    pub mfa_code: Option<MfaCode>,
}

#[derive(Debug, Serialize)]
pub struct AuthenticationResponse {
    pub user: User,
}

#[actix_web::post("/EmailPassword")]
pub(crate) async fn email_password(session: Session, Json(request): Json<EmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {
    
    // Creates the credential
    let credential = EmailPasswordMethod::new(request.email, request.password);

    // Fetches the user
    let user = credential
        .authenticate(&db, request.mfa_code)
        .await?;

    // Generates the ID token and saves it as a httpOnly cookie
    let id_token = user.get_id_token()?;
    session.insert("access_token", id_token.access)?;
    session.insert("refresh_token", id_token.refresh)?;

    Ok(Json(
        AuthenticationResponse {
            user,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/authenticate")
        .service(email_password)
}