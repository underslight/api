use actix_session::Session;
use actix_web::{web::{Data, Json}, Responder};
use auth::user::{attributes::UserAttributes, token::Token, User};
use surrealdb::{engine::remote::ws::Client, Surreal};
use serde::Serialize;

use crate::prelude::{ApiError, ApiResult};

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    user: User,
}

#[actix_web::put("/update")]
pub(crate) async fn update(session: Session, Json(request): Json<UserAttributes>, db: Data<Surreal<Client>>) -> ApiResult<impl Responder> {

    // Gets the access token from the session
    let access_token = session.get::<Token>("access_token")?
        .ok_or(ApiError::TokenInvalid)?;

    // Gets the user
    let mut user = User::get_by_access_token(&db, &access_token)
        .await?;

    user.attributes(request);

    // Updates the user
    user
        .update(&db)
        .await?;

    Ok(Json(
        UpdateResponse {
            user
        }
    ))
}