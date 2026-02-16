use crate::core::models::{AuthenticatedUser, UserResponse};
use crate::core::results::hello::GetHelloResult;
use crate::core::use_cases::hello::HelloQuery;
use crate::state::AppState;
use axum::extract::State;

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
pub async fn hello(State(state): State<AppState>) -> String {
    let mediator = &state.mediator;
    let result = mediator
        .query::<HelloQuery, GetHelloResult>(HelloQuery {
            name: "My name".to_string(),
        })
        .await;
    result.unwrap().name
}
