use crate::prelude::*;
use actix_identity::Identity;
use actix_web::{
    web::{self, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder, Scope,
};
use auth::{
    credential::{email_password::PartialEmailPassword, Credential, PartialCredential},
    user::User,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EmailPasswordAuthenticationRequest {
    pub email: String,
    pub password: String,
}

#[actix_web::post("/authenticate")]
pub async fn authenticate(
    _: DenyAuthenticated,
    http: HttpRequest,
    mut connection: Database,
    request: web::Json<EmailPasswordAuthenticationRequest>,
) -> Result<impl Responder> {
    let user = web::block::<_, Result<User>>(move || {
        let partial_credential =
            PartialEmailPassword::new(request.email.clone(), request.password.clone());
        User::authenticate(&mut connection, partial_credential).map_err(Error::from)
    })
    .await??;

    Identity::login(&http.extensions(), user.uid().to_string())?;

    Ok(HttpResponse::Ok().json(user))
}

#[derive(Deserialize)]
struct EmailPasswordRegistrationRequest {
    pub email: String,
    pub password: String,
}

#[actix_web::post("/register")]
pub async fn register(
    _: DenyAuthenticated,
    http: HttpRequest,
    mut connection: Database,
    request: web::Json<EmailPasswordRegistrationRequest>,
) -> Result<impl Responder> {
    let user = web::block::<_, Result<User>>(move || {
        let partial_credential =
            PartialEmailPassword::new(request.email.clone(), request.password.clone());
        User::new(&mut connection, partial_credential).map_err(Error::from)
    })
    .await??;

    Identity::login(&http.extensions(), user.uid().to_string())?;

    Ok(HttpResponse::Ok().json(user))
}

#[derive(Deserialize)]
struct EmailPasswordAssociationRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
struct EmailPasswordAssociationResponse {
    success: bool,
}

#[actix_web::post("/associate")]
pub async fn associate(
    user: RequireAuthenticated,
    mut connection: Database,
    request: Json<EmailPasswordAssociationRequest>,
) -> Result<impl Responder> {
    web::block::<_, Result<()>>(
        move || match user.credentials(&mut connection)?.email_password {
            Some(_) => Err(CredentialError::CredentialAssociated.into()),
            None => {
                PartialEmailPassword::new(request.email.clone(), request.password.clone())
                    .associate(&mut connection, user.uid())?;

                Ok(())
            }
        },
    )
    .await??;

    Ok(HttpResponse::Ok().json(EmailPasswordAssociationResponse { success: true }))
}

#[derive(Deserialize)]
struct EmailPasswordRemovalRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
struct EmailPasswordRemovalResponse {
    pub success: bool,
}

#[actix_web::delete("/remove")]
pub async fn remove(
    user: RequireAuthenticated,
    mut connection: Database,
    request: Json<EmailPasswordRemovalRequest>,
) -> Result<impl Responder> {
    web::block::<_, Result<()>>(move || {
        let partial_credential =
            PartialEmailPassword::new(request.email.clone(), request.password.clone());
        let credential = User::authenticate(&mut connection, partial_credential)?
            .credentials(&mut connection)?
            .email_password(&mut connection)?;

        if credential.uid() != user.uid() {
            return Err(CredentialError::CredentialIncorrect.into());
        }

        credential.delete(&mut connection).map_err(Error::from)
    })
    .await??;

    Ok(HttpResponse::Ok().json(EmailPasswordRemovalResponse { success: true }))
}

pub fn scope() -> Scope {
    web::scope("/email_password")
        .service(authenticate)
        .service(register)
        .service(associate)
        .service(remove)
}
