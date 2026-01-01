use service::settings::Settings;

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

    let app_ctx = service::ctx::application_ctx(&s)
        .await
        .expect("application ctx error");

    println!("project settings configured");

    service::tracer::init_open_telemetry_otlp(&s.telemetry);

    tracing::info!(message = "tracer configured");

    listener(app_ctx, s.service.bind, s.service.port)
        .await
        .expect("error in starting the server");

    Ok(())
}

pub async fn listener(ctx: base::Ctx, host: String, port: u16) -> std::io::Result<()> {
    // settings the opentelemetry protocol so jaeger can consume the logs

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
