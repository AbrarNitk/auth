pub mod auth;

pub fn routes<S: Send + Sync + 'static + Clone>(app_context: base::Ctx) -> axum::Router<S> {
    auth::authn_router(app_context)
}
