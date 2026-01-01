pub fn authn_router<S: Send + Sync + 'static + Clone>(app_context: base::Ctx) -> axum::Router<S> {
    axum::Router::new()
        .route(
            "/v2/api/authn/login",
            axum::routing::get(oidc::controller::login_with),
        )
        .route(
            "/v2/api/authn/callback",
            axum::routing::get(oidc::controller::authn_callback),
        )
        .with_state(app_context)
        .layer(crate::middleware::client_info::ReqClientInfoLayer)
}
