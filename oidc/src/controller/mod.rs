pub mod errors;

use axum::{
    Extension,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};

#[derive(Debug, serde::Deserialize)]
pub struct LoginReqQuery {
    pub with: String,
    #[serde(alias = "next-url")]
    pub next_url: Option<String>,
    #[serde(alias = "api", default)]
    pub is_api: bool,
}
#[tracing::instrument(name = "authn::login-with", skip_all, parent = None)]
pub async fn login_with(
    State(ctx): State<base::Ctx>,
    Extension(client_info): Extension<base::ReqClientInfo>,
    Query(query): Query<LoginReqQuery>,
) -> Response {
    todo!()
}

pub async fn authn_callback() -> Response {
    todo!()
}
