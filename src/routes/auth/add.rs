use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::{email_password::EmailPasswordMethod, totp::TotpMethod}, token::Token, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct AddEmailPasswordRequest {
    pub email: String,
    pub password: String,
    pub access_token: Token,
}

#[derive(Debug, Deserialize)]
pub struct AddTotpRequest {
    pub access_token: Token,
}

#[derive(Debug, Serialize)]
pub struct AddAuthMethodResponse {
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct AddMfaMethodResponse {
    pub user: User,
    pub data: String,
}

#[actix_web::post("/EmailPassword")]
pub(crate) async fn email_password(Json(request): Json<AddEmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {
    
    // Fetches the user
    let user = User::get_by_access_token(&db, &request.access_token)
        .await?;
 
    // Creates the credential
    let credential = EmailPasswordMethod::new(request.email, request.password);

    // Adds the authentication method
    user
        .add_auth_method(&db, Box::new(credential))
        .await?;

    Ok(Json(
        AddAuthMethodResponse {
            user,
        }
    ))
}

#[actix_web::post("/Totp")]
pub(crate) async fn totp(Json(request): Json<AddTotpRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Fetches the user
    let user = User::get_by_access_token(&db, &request.access_token)
        .await?;

    // Creates the Totp credential
    let secret = "salkdjaskljd".to_string();
    let mfa_credential = TotpMethod::new(user.id, secret.clone());

    // Adds the MFA method
    user
        .add_mfa_method(&db, Box::new(mfa_credential))
        .await?;

    Ok(Json(
        AddMfaMethodResponse {
            user,
            data: secret,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/add")
        .service(email_password)
        .service(totp)
}