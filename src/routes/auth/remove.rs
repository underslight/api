use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::{AuthMethodType, MfaMethodType}, token::Token, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct RemoveEmailPasswordRequest {
    pub email: String,
    pub password: String,
    pub access_token: Token,
}

#[derive(Debug, Deserialize)]
pub struct RemoveTotpRequest {
    pub access_token: Token,
}

#[derive(Debug, Serialize)]
pub struct RemoveMethodResponse {
    pub user: User,
}

#[actix_web::delete("/EmailPassword")]
pub(crate) async fn email_password(Json(request): Json<RemoveEmailPasswordRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {
    
    // Fetches the user
    let user = User::get_by_access_token(&db, &request.access_token)
        .await?;

    // Adds the authentication method
    user
        .remove_auth_method(&db, AuthMethodType::EmailPassword)
        .await?;

    Ok(Json(
        RemoveMethodResponse {
            user,
        }
    ))
}

#[actix_web::delete("/Totp")]
pub(crate) async fn totp(Json(request): Json<RemoveTotpRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Fetches the user and removes the MFA method
    let user = User::get_by_access_token(&db, &request.access_token)
        .await?
        .remove_mfa_method(&db, MfaMethodType::Totp)
        .await?;

    Ok(Json(
        RemoveMethodResponse {
            user,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/remove")
        .service(email_password)
        .service(totp)
}