use actix_session::Session;
use actix_web::{web::{self, Data, Json}, Responder, Scope};
use auth::user::{credential::{AuthMethodType, MfaMethodType}, token::Token, User};
use serde::{Serialize, Deserialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct UnlinkAuthMethodRequest {
    pub auth_method: AuthMethodType,
}

#[derive(Debug, Deserialize)]
pub struct UnlinkMfaMethodRequest {
    pub mfa_method: MfaMethodType,
}

#[derive(Debug, Serialize)]
pub struct UnlinkMethodResponse {
    pub user: User,
}

#[actix_web::delete("/auth")]
pub(crate) async fn auth_method(session: Session, Json(request): Json<UnlinkAuthMethodRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {
    
    // Fetches the access token from the cookies
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Fetches the user
    let user = User::get_by_access_token(&db, &access_token)
        .await?;

    // Adds the authentication method
    user
        .remove_auth_method(&db, request.auth_method)
        .await?;

    Ok(Json(
        UnlinkMethodResponse {
            user,
        }
    ))
}

#[actix_web::delete("/mfa")]
pub(crate) async fn mfa_method(session: Session, Json(request): Json<UnlinkMfaMethodRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Fetches the access token from the cookies
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Fetches the user and removes the MFA method
    let user = User::get_by_access_token(&db, &access_token)
        .await?
        .remove_mfa_method(&db, request.mfa_method)
        .await?;

    Ok(Json(
        UnlinkMethodResponse {
            user,
        }
    ))
}

pub fn scope() -> Scope {
    web::scope("/unlink")
        .service(auth_method)
        .service(mfa_method)
}