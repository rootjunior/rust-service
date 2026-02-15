use crate::core::models::AuthResult;
use crate::core::models::UserResponse;
use api::v1::handlers::__path_hello;
use api::v1::handlers::__path_me;

use crate::api;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "User API",
        version = "1.0.0",
        description = "API для управления пользователями с аутентификацией",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    paths(
        me,
        hello
    ),
    components(schemas(
        UserResponse,
        AuthResult
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(
                            utoipa::openapi::security::HttpAuthScheme::Bearer,
                        )
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
// const API_KEY_NAME: &str = "Some-Auth-Key";
// components.add_security_scheme(
//     API_KEY_NAME,
//     SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
//         API_KEY_NAME,
//     ))),
// );
