use crate::core::handlers::hello::HelloQuery;
use crate::core::models::{AuthenticatedUser, UserResponse};
use crate::core::results::hello::GetHelloResult;
use crate::mediator::mediator::Mediator;
use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/v1/me",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Информация о текущем пользователе", body = UserResponse),
        (status = 401, description = "Не авторизован: отсутствует или неверный токен")
    )
)]
pub async fn me(user: AuthenticatedUser) -> UserResponse {
    UserResponse { user_id: user.0.user_id }
}

#[utoipa::path(
    get,
    path = "/api/v1/hello",
    tag = "Hello",
    responses(
        (status = 200, description = "Приветственное сообщение")
    )
)]
pub async fn hello(State(mediator): State<Arc<Mediator>>) -> String {
    // Обрабатывать ошибки
    mediator
        .query::<HelloQuery, GetHelloResult>(HelloQuery {
            name: "My name".to_string(),
        })
        .await
        .unwrap()
        .name
}
