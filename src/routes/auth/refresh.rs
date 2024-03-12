use actix_session::Session;
use actix_web::{web::{Data, Json}, Responder};
use auth::user::token::{IdToken, Token};
use serde::Serialize;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::prelude::{ApiError, ApiResult};

#[derive(Debug, Serialize)]
pub(crate) struct RefreshResponse {
    pub success: bool,
}

#[actix_web::post("/refresh")]
pub(crate) async fn refresh(session: Session, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Creates the ID token
    let id_token = IdToken {
        access: session.get::<Token>("access_token")?
            .ok_or(ApiError::TokenInvalid)?,
        refresh: session.get::<Token>("refresh_token")?
            .ok_or(ApiError::TokenInvalid)?
    };

    // Refreshes the ID token
    let id_token = id_token.refresh(&db).await?;

    session.insert("access_token", id_token.access)?;
    session.insert("refresh_token", id_token.refresh)?;

    // Refreshes the id token
    Ok(Json(
        RefreshResponse {
            success: true,
        }
    ))
}