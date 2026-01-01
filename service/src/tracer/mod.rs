pub fn init_open_telemetry_otlp(_settings: &base::ctx::settings::telemetry::TelemetrySettings) {
    use tracing_subscriber::prelude::*;

    // Temporarily disabled OpenTelemetry tracing due to version compatibility issues
    // let telemetry_layer = create_otlp_tracer(settings).map(|tracer| {
    //     tracing_opentelemetry::layer()
    //         .with_location(false)
    //         .with_tracer(tracer)
    // });

    let formatting_layer = tracing_bunyan_formatter::BunyanFormattingLayer::new(
        "tracing_demo".into(),
        std::io::stdout,
    );

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from(
            "info",
            //settings.log_level.as_str(),
        ))
        // Forest Layer
        .with(tracing_forest::ForestLayer::default())
        // bunyan Json Storage Layer
        .with(tracing_bunyan_formatter::JsonStorageLayer)
        .with(formatting_layer)
        // simple stdout layer
        // .with(
        //     tracing_subscriber::fmt::layer()
        //         .with_ansi(std::io::stderr().is_terminal())
        //         .with_target(false)
        //         .with_line_number(true)
        //         .with_file(false)
        //         .without_time(),
        // )
        // telemetry layer - temporarily disabled
        // .with(telemetry_layer)
        .init();
}
