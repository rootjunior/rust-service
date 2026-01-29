use crate::configs::AppState;
use crate::models::{AuthResult, AuthenticatedUser};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(_state);

        // Берём заголовок Authorization
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing token"))?
            .to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid token format"))?;
        // Проверка токена
        if token != app_state.cfg.server_address {
            return Err((StatusCode::UNAUTHORIZED, "Invalid token"));
        }

        Ok(AuthenticatedUser(AuthResult { user_id: 42 }))
    }
}
