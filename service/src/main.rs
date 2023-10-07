pub struct HttpService {
    pool: db::pg::DbPool,
}

impl hyper::service::Service<hyper::Request<hyper::Body>> for HttpService {
    type Response = hyper::Response<hyper::Body>;
    type Error = hyper::Error;
    type Future = std::pin::Pin<
        Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
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

fn read_env() -> String {
    match std::env::var("ENV") {
        Ok(env) => env.to_lowercase(),
        Err(_) => "local".to_string(),
    }
}

async fn http_main() -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
    // Setting the environment variables
    let env_path = format!("{}.env", read_env());
    dotenv::from_path(env_path.as_str()).ok();
    tracing::info!("Environment set: {}", env_path);

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var not found");
    // Initializing the database pool
    // db::redis::init_redis_pool();

    // Creating the tcp listener
    let socket_address: std::net::SocketAddr = ([0, 0, 0, 0], 8001).into();
    let listener = tokio::net::TcpListener::bind(socket_address).await?;
    tracing::info!(
        "#### Started at: {}:{} ####",
        socket_address.ip(),
        socket_address.port()
    );
    let db_url = db_url.clone();
    let pool = db::pg::get_connection_pool(db_url.as_str());
    loop {
        let pool = pool.clone();
        let (tcp_stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(http_err) = hyper::server::conn::Http::new()
                .serve_connection(tcp_stream, HttpService { pool })
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
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
