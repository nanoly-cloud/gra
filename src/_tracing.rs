pub fn init() {
    let registry = tracing_subscriber::registry::Registry::default()
        .with(EnvFilter::from_env("GRA_LOG"))
        .with(
            tracing_subscriber::fmt::layer().pretty(),
            // .json(),
            // .with_span_events(FmtSpan::CLOSE),
            // .with_target(false)
            // .with_thread_names(false)
            // .with_timer(tracing_subscriber::fmt::time::SystemTime),
            //
            // ??? .with_ansi(true)
        );

    #[cfg(feature = "tracing-forest")]
    registry.with(ForestLayer::default()).init();

    #[cfg(not(feature = "tracing-forest"))]
    {
        let format = tracing_subscriber::fmt::layer()
            .pretty()
            .with_level(false)
            .with_target(false)
            .with_thread_names(false)
            .with_span_events(FmtSpan::CLOSE)
            .with_ansi(true)
            .with_timer(tracing_subscriber::fmt::time::SystemTime)
            .compact();

        registry.with(format).init();
    }

    debug!("Initialised Tracing");
}
