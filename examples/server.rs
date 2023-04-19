use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // register global runtime collector
    prometheus::default_registry()
        .register(Box::new(
            tokio_metrics_collector::default_runtime_collector(),
        ))
        .unwrap();

    // register global task collector
    let task_collector = tokio_metrics_collector::default_task_collector();
    prometheus::default_registry()
        .register(Box::new(task_collector))
        .unwrap();

    // create a monitor for collecting `user` metrics
    let monitor_get_user = tokio_metrics::TaskMonitor::new();
    task_collector.add("get_user", monitor_get_user.clone());

    // create app
    let app = Router::new()
        .route("/", get(root))
        .route("/send_email", get(send_email))
        .route(
            "/user",
            axum::routing::get({
                let monitor = monitor_get_user.clone();
                move || {
                    monitor.instrument(async {
                        // Get user from database
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        r#"{"name": "miho"}"#
                    })
                }
            }),
        )
        .route("/metrics", get(metrics));

    // bind and serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn send_email() -> &'static str {
    let monitor = tokio_metrics::TaskMonitor::new();
    let task_collector = tokio_metrics_collector::default_task_collector();
    task_collector.add("email", monitor.clone());

    // Background task to send an email
    tokio::spawn(monitor.clone().instrument(async {
        tokio::time::sleep(Duration::from_secs(2)).await;
    }));
    "Email is sent"
}

async fn metrics() -> Result<String, String> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::default_registry().gather(), &mut buffer) {
        return Err(format!("could not encode custom metrics: {e}"));
    };
    String::from_utf8(buffer.clone())
        .map_err(|e| format!("custom metrics could not be from_utf8'd: {e}"))
}
