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
    pub partner: String,
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
    let req_id = "req_id".to_string();

    tracing::info!(msg = "authn:login", req_id = req_id, client_info = ?client_info);
    match crate::service::login_with::signin_url(
        &ctx,
        &client_info,
        &query.partner,
        &query.next_url,
    )
    .await
    {
        Ok(result) => {
            if query.is_api {
                // response as JSON
                base::response::success::success(result)
            } else {
                // response as redirect 302 Found
                (
                    axum::http::StatusCode::FOUND,
                    [(axum::http::header::LOCATION, result.uri)],
                )
                    .into_response()
            }
        }
        Err(err) => {
            tracing::error!(msg = "authn-login-url-error", err=?err);
            base::response::errors::error(req_id.to_string().as_str(), err)
        }
    }
}

pub async fn authn_callback() -> Response {
    todo!()
}
