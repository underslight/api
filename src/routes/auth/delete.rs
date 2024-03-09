use actix_web::{web::{Data, Json}, Responder};
use auth::user::{token::Token, User};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub(crate) struct DeleteRequest {
    pub access_token: Token,
}

#[derive(Debug, Serialize)]
pub(crate) struct DeleteResponse {
    pub success: bool,
}

#[actix_web::delete("/delete")]
pub(crate) async fn delete(Json(request): Json<DeleteRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Fetches the user
    let user = User::get_by_access_token(&db, &request.access_token)
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