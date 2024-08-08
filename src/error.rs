use std::{fmt::Display, ops::Deref};

use actix_web::{HttpResponse, ResponseError};
use oauth2::http::StatusCode;
use serde::Serialize;
use thiserror::Error as ErrorTrait;

pub trait ErrorKind
where
    Self: std::error::Error + ErrorCode + ResponseError + Send,
{
}

pub trait ErrorCode {
    fn error_code(&self) -> String;
}

#[derive(ErrorTrait, Debug)]
pub struct Error(Box<dyn ErrorKind>);
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn new(error: impl ErrorKind + 'static) -> Self {
        Self(Box::new(error))
    }
}

impl ErrorCode for &Error {
    fn error_code(&self) -> String {
        self.0.error_code()
    }
}

impl<T: ErrorKind + 'static> From<T> for Error {
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

impl Deref for Error {
    type Target = Box<dyn ErrorKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        (**self).status_code()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(ErrorResponse::from(self))
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl<T: std::error::Error + ErrorCode> From<T> for ErrorResponse {
    fn from(value: T) -> Self {
        Self {
            code: value.error_code(),
            message: value.to_string(),
        }
    }
}

#[derive(Serialize, ErrorTrait, Debug, Clone)]
pub enum UserError {
    #[error("The user is already registered!")]
    UserExists,
    #[error("The user is already authenticated!")]
    UserAuthenticated,
    #[error("The user doesn\'t exist or couldn\'t be found!")]
    UserNotFound,
    #[error("{0}")]
    Unknown(String),
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UserExists => StatusCode::CONFLICT,
            Self::UserAuthenticated => StatusCode::FORBIDDEN,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(ErrorResponse::from(self.clone()))
    }
}

impl ErrorCode for UserError {
    fn error_code(&self) -> String {
        match self {
            Self::UserNotFound => "AUTH/USER_NOT_FOUND",
            Self::UserAuthenticated => "AUTH/USER_AUTHENTICATED",
            Self::UserExists => "AUTH/USER_EXISTS",
            Self::Unknown(_) => "ERROR/UNKNOWN",
        }
        .to_string()
    }
}

impl ErrorKind for UserError {}

#[derive(Serialize, ErrorTrait, Debug, Clone)]
pub enum CredentialError {
    #[error("The credentials are incorrect!")]
    CredentialIncorrect,
    #[error("Failed to connect to auth provider!")]
    OauthCodeIncorrect,
    #[error("The credential is disabled!")]
    CredentialDisabled,
    #[error("The authentication method is already associated!")]
    CredentialAssociated,
    #[error("Cannot remove the only associated authentication method!")]
    CredentialCannotRemove,
    #[error("A credential is required to perform this operation!")]
    CredentialRequired,
    #[error("{0}")]
    Unknown(String),
}

impl ResponseError for CredentialError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::CredentialIncorrect | Self::CredentialRequired | Self::OauthCodeIncorrect => {
                StatusCode::UNAUTHORIZED
            }
            Self::CredentialAssociated | Self::CredentialCannotRemove => StatusCode::CONFLICT,
            Self::CredentialDisabled => StatusCode::FORBIDDEN,
            Self::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(ErrorResponse::from(self.clone()))
    }
}

impl ErrorCode for CredentialError {
    fn error_code(&self) -> String {
        match self {
            Self::CredentialAssociated => "AUTH/CREDENTIAL_ASSOCIATED",
            Self::CredentialCannotRemove => "AUTH/CREDENTIAL_CANNOT_REMOVE",
            Self::CredentialDisabled => "AUTH/CREDENTIAL_DISABLED",
            Self::CredentialIncorrect => "AUTH/CREDENTIAL_INCORRECT",
            Self::CredentialRequired => "AUTH/CREDENTIAL_REQUIRED",
            Self::OauthCodeIncorrect => "AUTH/OAUTH_CODE_INCORRECT",
            Self::Unknown(_) => "ERROR/UNKNOWN",
        }
        .to_string()
    }
}

impl ErrorKind for CredentialError {}

#[derive(Serialize, ErrorTrait, Debug, Clone)]
pub enum ApiError {
    #[error("The resource couldn\'t be found ot doesn\'t exist!")]
    ResourceNotFound,
    #[error("{0}")]
    Unknown(String),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ResourceNotFound => StatusCode::NOT_FOUND,
            Self::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(ErrorResponse::from(self.clone()))
    }
}

impl ErrorCode for ApiError {
    fn error_code(&self) -> String {
        match self {
            Self::ResourceNotFound => "API/RESOURCE_NOT_FOUND",
            Self::Unknown(_) => "ERROR/UNKNOWN",
        }
        .to_string()
    }
}

impl ErrorKind for ApiError {}

impl From<auth::error::AuthError> for Error {
    fn from(value: auth::error::AuthError) -> Self {
        use auth::error::AuthError;

        match value {
            AuthError::CredentialDisabled => CredentialError::CredentialDisabled.into(),
            AuthError::NotFound(_) => UserError::UserNotFound.into(),
            AuthError::Exists(_) => UserError::UserExists.into(),
            AuthError::CredentialCannotDelete => CredentialError::CredentialCannotRemove.into(),
            AuthError::Hash | AuthError::Database(_) => {
                CredentialError::Unknown("Something went wrong!".into()).into()
            }
            AuthError::Unknown(message) | AuthError::Invalid(message) => {
                UserError::Unknown(message).into()
            }
        }
    }
}

impl From<oauth2::url::ParseError> for Error {
    fn from(_value: oauth2::url::ParseError) -> Self {
        ApiError::Unknown("The URL is incorrect or invalid!".into()).into()
    }
}

impl From<r2d2::Error> for Error {
    fn from(_value: r2d2::Error) -> Self {
        ApiError::Unknown("Something went wrong!".into()).into()
    }
}

impl From<actix_web::error::BlockingError> for Error {
    fn from(_value: actix_web::error::BlockingError) -> Self {
        ApiError::Unknown("Something went wrong!".into()).into()
    }
}

impl From<actix_identity::error::LoginError> for Error {
    fn from(_value: actix_identity::error::LoginError) -> Self {
        ApiError::Unknown("Failed to authenticate!".into()).into()
    }
}

impl From<actix_identity::error::GetIdentityError> for Error {
    fn from(_value: actix_identity::error::GetIdentityError) -> Self {
        ApiError::Unknown("Failed to authenticate!".into()).into()
    }
}

impl From<uuid::Error> for Error {
    fn from(_value: uuid::Error) -> Self {
        UserError::UserNotFound.into()
    }
}
