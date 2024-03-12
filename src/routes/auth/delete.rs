use actix_session::Session;
use actix_web::{web::{Data, Json}, Responder};
use auth::user::{token::Token, User};
use serde::Serialize;
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Serialize)]
pub(crate) struct DeleteResponse {
    pub success: bool,
}

#[actix_web::delete("/delete")]
pub(crate) async fn delete(session: Session, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Gets the user's access token
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Fetches the user
    let user = User::get_by_access_token(&db, &access_token)
        .await?;

    // Deletes the user
    user
        .delete(&db)
        .await?;

    Ok(Json(
        DeleteResponse {
            success: true
        }
    ))
}