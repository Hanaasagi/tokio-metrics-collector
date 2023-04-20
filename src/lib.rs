#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

//! Provides utilities for collecting Prometheus-compatible metrics from Tokio runtime and tasks.
//! You can find metrics definition [here](https://github.com/Hanaasagi/tokio-metrics-collector).
//!
//! ### Collecting task metrics
//! In the below example, a [`TaskCollector`] is constructed and used to
//! instrument a simple task. Task metrics is collected every second.
//! ```
//! use prometheus::Encoder;
//!
//! #[tokio::main]
//! async fn main() {
//!     // register global task collector
//!     let task_collector = tokio_metrics_collector::default_task_collector();
//!     prometheus::default_registry()
//!         .register(Box::new(task_collector))
//!         .unwrap();
//!
//!     // construct a TaskMonitor
//!     let monitor = tokio_metrics_collector::TaskMonitor::new();
//!     // add this monitor to task collector with label 'simple_task'
//!     task_collector.add("simple_task", monitor.clone());
//!
//!     // spawn a background task and instrument
//!     tokio::spawn(monitor.instrument(async {
//!         loop {
//!             // do something here
//!             tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
//!         }
//!     }));
//!
//!     // print metrics every tick
//!     for _ in 0..5 {
//!         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//!
//!         let encoder = prometheus::TextEncoder::new();
//!         let mut buffer = Vec::new();
//!         encoder
//!             .encode(&prometheus::default_registry().gather(), &mut buffer)
//!             .unwrap();
//!         let data = String::from_utf8(buffer.clone()).unwrap();
//!
//!         println!("{}", data);
//!     }
//! }
//! ```

#![cfg_attr(
    all(tokio_unstable, feature = "rt"),
    doc = r##"
### Collecting runtime metrics
**This functionality requires `tokio_unstable` and the crate feature `rt`.**

In the below example, a [`RuntimeCollector`] is constructed and used to
collect tokio runtime metrics every second.
```
use prometheus::Encoder;

#[tokio::main]
async fn main() {
    // register global runtime collector
    prometheus::default_registry()
        .register(Box::new(
            tokio_metrics_collector::default_runtime_collector(),
        ))
        .unwrap();

    // print metrics every tick
    for _ in 0..5 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&prometheus::default_registry().gather(), &mut buffer)
            .unwrap();
        let data = String::from_utf8(buffer.clone()).unwrap();

        println!("{}", data);
    }
}
```
"##
)]

mod task;

pub use task::default_collector as default_task_collector;
pub use task::TaskCollector;
pub use tokio_metrics::TaskMonitor;

macro_rules! cfg_rt {
    ($($item:item)*) => {
        $(
            #[cfg(all(tokio_unstable, feature = "rt"))]
            #[cfg_attr(docsrs, doc(cfg(all(tokio_unstable, feature = "rt"))))]
            $item
        )*
    };
}

cfg_rt! {
    mod runtime;
    pub use runtime::default_collector as default_runtime_collector;
    pub use runtime::RuntimeCollector;
}
