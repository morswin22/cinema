use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tower_sessions::Session;
use crate::models::User; // Adjust path to your User struct
use crate::{SESSION_USER_KEY, AppError};
use axum::response::Response;

pub struct OptionalUser(pub Option<User>);

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::UnauthorizedError)?;

        let user = session.get::<User>(SESSION_USER_KEY)
            .await
            .map_err(|_| AppError::UnauthorizedError)?;

        Ok(OptionalUser(user))
    }
}


pub struct RequiredUser(pub User);

impl<S> FromRequestParts<S> for RequiredUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let OptionalUser(user) = OptionalUser::from_request_parts(parts, state).await?;
        user.ok_or(AppError::UnauthorizedError).map(RequiredUser)
    }
}
