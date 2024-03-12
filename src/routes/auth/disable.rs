use actix_session::Session;
use actix_web::{web::{Data, Json}, Responder};
use auth::user::{token::Token, User};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::prelude::{ApiError, ApiResult};

#[derive(Debug, Deserialize)]
pub struct DisableRequest {
    pub reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DisableResponse {
    pub success: bool,
}

#[actix_web::put("/disable")]
pub(crate) async fn disable(session: Session, Json(request): Json<DisableRequest>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Gets the access token
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Gets the user
    let mut user = User::get_by_access_token(&db, &access_token)
        .await?;

    // Updates the user
    user 
        .disabled(Some(request.reasons))
        .update(&db)
        .await?;

    Ok(Json(
        DisableResponse {
            success: true,
        }
    ))
}