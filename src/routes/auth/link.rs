use actix_session::Session;
use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::{email_password::EmailPasswordMethod, totp::TotpMethod, MfaMethodType}, token::Token, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct LinkEmailPasswordRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkMfaRequest {
    mfa_method: MfaMethodType,
}

#[derive(Debug, Serialize)]
pub struct LinkAuthMethodResponse {
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct LinkMfaMethodResponse {
    pub user: User,
    pub data: String,
}

#[actix_web::post("/EmailPassword")]
pub(crate) async fn email_password(session: Session, Json(request): Json<LinkEmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {
    
    // Fetches the access token from the cookie
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Fetches the user
    let user = User::get_by_access_token(&db, &access_token)
        .await?;
 
    // Creates the credential
    let credential = EmailPasswordMethod::new(request.email, request.password);

    // Adds the authentication method
    user
        .add_auth_method(&db, Box::new(credential))
        .await?;

    Ok(Json(
        LinkAuthMethodResponse {
            user,
        }
    ))
}

#[actix_web::post("/mfa")]
pub(crate) async fn mfa(session: Session, Json(request): Json<LinkMfaRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Fetches the access token from the cookies
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Fetches the user
    let user = User::get_by_access_token(&db, &access_token)
        .await?;

    // Creates the Totp credential
    let secret = "salkdjaskljd".to_string();
    let mfa_credential = match request.mfa_method {
        MfaMethodType::Totp => TotpMethod::new(user.id, secret.clone()),
    };

    // Adds the MFA method
    user
        .add_mfa_method(&db, Box::new(mfa_credential))
        .await?;

    Ok(Json(
        LinkMfaMethodResponse {
            user,
            data: secret,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/link")
        .service(email_password)
        .service(mfa)
}