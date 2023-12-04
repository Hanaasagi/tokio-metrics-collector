use axum::extract::State;
use axum::{routing::get, Router};
use std::time::Duration;
use tokio_metrics::TaskMonitor;

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
    let monitor_get_user = tokio_metrics_collector::TaskMonitor::new();
    task_collector
        .add("get_user", monitor_get_user.clone())
        .unwrap();

    // create a monitor for collecting `send_email` metrics
    let email_monitor = tokio_metrics_collector::TaskMonitor::new();
    task_collector.add("email", email_monitor.clone()).unwrap();

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
        .route("/metrics", get(metrics))
        // share the monitor with routes
        .with_state(email_monitor);

    // bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn send_email(State(email_monitor): State<TaskMonitor>) -> &'static str {
    // Background task to send an email
    tokio::spawn(email_monitor.clone().instrument(async {
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
