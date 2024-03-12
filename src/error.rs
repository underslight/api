use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("USER_NOT_FOUND")]
    UserNotFound(String),
    #[error("USER_DISABLED")]
    UserDisabled(Vec<String>),
    #[error("HASH_FAILED")]
    HashFailed,
    #[error("CREDENTIAL_DUPLICATE")]
    CredentialDuplicate(String),
    #[error("CREDENTIAL_ONLY")]
    CredentialOnly(String),
    #[error("CREDENTIAL_NOT_FOUND")]
    CredentialNotFound(String),
    #[error("MFA_REQUIRED")]
    MfaRequired,
    #[error("IO")]
    Io(std::io::Error),
    #[error("SAVE_FAILED")]
    SaveFailed(String),
    #[error("UPDATE_FAILED")]
    UpdateFailed(String),
    #[error("TOKEN_EXPIRED")]
    TokenExpired,
    #[error("TOKEN_INVALID")]
    TokenInvalid,
    #[error("DATABASE_OP_FAILED")]
    DatabaseFailed(surrealdb::Error),    
    #[error("UNKNOWN")]
    Unknown(String),
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub message: String,
    pub code: String,
}

impl From<&ApiError> for ApiErrorResponse {
    fn from(value: &ApiError) -> Self {
        Self {
            message: match value {
                ApiError::UserNotFound(m) 
                | ApiError::CredentialDuplicate(m)
                | ApiError::CredentialOnly(m)
                | ApiError::CredentialNotFound(m)
                | ApiError::SaveFailed(m)
                | ApiError::UpdateFailed(m)
                | ApiError::Unknown(m) => m.to_string(),
                ApiError::MfaRequired => "MFA is required to access this account!".into(),
                ApiError::UserDisabled(reason) => format!("Your account has been disabled for the following reason(s): {}", reason.join(", ")),
                ApiError::TokenExpired => "The ID token has expired!".into(),
                ApiError::TokenInvalid => "The ID token is invalid!".into(),
                ApiError::HashFailed 
                | ApiError::Io(_)
                | ApiError::DatabaseFailed(_) => "Something went wrong!".into(),
            },
            code : format!("{}", value),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(ApiErrorResponse::from(self))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::UserNotFound(_) 
            | ApiError::UserDisabled(_)
            | ApiError::CredentialNotFound(_)
            | ApiError::TokenExpired
            | ApiError::MfaRequired
            | ApiError::TokenInvalid => StatusCode::FORBIDDEN,
            ApiError::HashFailed 
            | ApiError::Io(_)
            | ApiError::SaveFailed(_)
            | ApiError::UpdateFailed(_)
            | ApiError::DatabaseFailed(_)
            | ApiError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::CredentialDuplicate(_)
            | ApiError::CredentialOnly(_) => StatusCode::BAD_REQUEST
        }
    }
} 

impl From<auth::error::AuthError> for ApiError {
    fn from(value: auth::error::AuthError) -> Self {
        use auth::error::AuthError;
        
        match value {
            AuthError::UserNotFound(message) => Self::UserNotFound(message),
            AuthError::UserDisabled(reason) => Self::UserDisabled(reason),
            AuthError::HashFailed => Self::HashFailed,
            AuthError::CredentialOnly(message) => Self::CredentialOnly(message),
            AuthError::CredentialDuplicate(message) => Self::CredentialDuplicate(message),
            AuthError::CredentialNotFound(message) => Self::CredentialNotFound(message),
            AuthError::MfaRequired => Self::MfaRequired,
            AuthError::Io(error) => Self::Io(error), 
            AuthError::SaveFailed(message) => Self::SaveFailed(message),
            AuthError::UpdateFailed(message) => Self::UpdateFailed(message),
            AuthError::DatabaseFailed(error) => Self::DatabaseFailed(error),
            AuthError::TokenExpired => Self::TokenExpired,
            AuthError::TokenInvalid => Self::TokenInvalid,
            AuthError::Unknown(panic) => Self::Unknown(panic),
            _ => Self::Unknown("Something went wrong!".into()),
        }
    }
}

impl From<SessionGetError> for ApiError {
    fn from(_value: SessionGetError) -> Self {
        Self::Unknown("Failed to authenticate!".into())
    }
}

impl From<SessionInsertError> for ApiError {
    fn from(_value: SessionInsertError) -> Self {
        Self::Unknown("Failed to authenticate!".into())
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;