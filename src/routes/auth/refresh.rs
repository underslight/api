use actix_web::{web::{Data, Json}, Responder};
use auth::user::token::IdToken;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::prelude::ApiResult;

#[derive(Debug, Deserialize)]
pub(crate) struct RefreshRequest {
    pub id_token: IdToken,
}

#[derive(Debug, Serialize)]
pub(crate) struct RefreshResponse {
    pub id_token: IdToken,
}

#[actix_web::post("/refresh")]
pub(crate) async fn refresh(Json(request): Json<RefreshRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Refreshes the id token
    Ok(Json(
        RefreshResponse {
            id_token: request.id_token.refresh(&db).await?
        }
    ))
}