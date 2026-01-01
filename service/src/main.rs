use hyper::body::Incoming;
use service::settings::Settings;

pub struct HttpService {
    pool: db::pg::DbPool,
}

impl hyper::service::Service<hyper::Request<Incoming>> for HttpService {
    type Response = hyper::Response<Vec<u8>>;
    type Error = hyper::Error;
    type Future = std::pin::Pin<
        Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn call(&self, req: hyper::Request<Incoming>) -> Self::Future {
        let pool = self.pool.clone();
        Box::pin(async move {
            match service::route::handler(req, pool).await {
                Ok(r) => Ok(r),
                Err(e) => {
                    tracing::error!(message = "error:route::handler", error = format!("{e}"));
                    Ok(service::controller::response(
                        serde_json::json!({"message": "Internal Server Error","success": false})
                            .to_string(),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        })
    }
}

pub fn read_env() -> String {
    match std::env::var("ENV") {
        Ok(env) => env.to_lowercase(),
        Err(_) => "local".to_string(),
    }
}
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(http_main())
        .unwrap();
}

pub async fn http_main() -> std::io::Result<()> {
    let s = Settings::new_with_file("etc/settings", "dev").expect("wrong settings");
    println!("settings: {:?}", s);
    Ok(())
}

pub async fn listener(ctx: base::Ctx, host: String, port: u16) -> std::io::Result<()> {
    // settings the opentelemetry protocol so jaeger can consume the logs

    tracing::info!(message = "project settings configured");

    //Creating the listener on provided bind address and port
    let listener =
        tokio::net::TcpListener::bind(std::net::SocketAddr::new(host.parse().unwrap(), port))
            .await?;

    tracing::info!(
        message = "#### Server Started ####",
        bind = listener.local_addr().unwrap().ip().to_string(),
        port = listener.local_addr().unwrap().port()
    );

    let router = service::routes::routes(ctx);
    // let router = router.nest_service("/", ServeDir::new("./frontend/dist"));

    //Starting the application on the listener with the application router
    axum::serve(listener, router)
        // .with_graceful_shutdown(shutdown_signal)
        .await?;
    // services::tracer::opob::shutdown_tracer();
    Ok(())
}
