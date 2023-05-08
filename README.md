# Tokio metrics collector for Prometheus

[![tokio-metrics-collector on GitHub Action][action-image]][action]
[![codecov][codecov-image]][codecov]
[![tokio-metrics-collector on crates.io][cratesio-image]][cratesio]
[![tokio-metrics-collector on docs.rs][docsrs-image]][docsrs]

[action-image]: https://github.com/Hanaasagi/tokio-metrics-collector/actions/workflows/ci.yaml/badge.svg
[action]: https://github.com/Hanaasagi/tokio-metrics-collector/actions/workflows/ci.yaml
[codecov-image]: https://codecov.io/gh/Hanaasagi/tokio-metrics-collector/branch/master/graph/badge.svg?token=TP67FTFT1K
[codecov]: https://codecov.io/gh/Hanaasagi/tokio-metrics-collector
[cratesio-image]: https://img.shields.io/crates/v/tokio-metrics-collector.svg
[cratesio]: https://crates.io/crates/tokio-metrics-collector
[docsrs-image]: https://docs.rs/tokio-metrics-collector/badge.svg
[docsrs]: https://docs.rs/tokio-metrics-collector

Provides utilities for collecting Prometheus-compatible metrics from Tokio runtime and tasks.

```toml
[dependencies]
tokio-metrics-collector = { version = "0.1.0" }
```

Documentation:

- [API reference (docs.rs)](https://docs.rs/tokio-metrics-collector/)

## QuickStart

```Rust
use prometheus::Encoder;

#[tokio::main]
async fn main() {
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

    // construct a TaskMonitor
    let monitor = tokio_metrics_collector::TaskMonitor::new();
    // add this monitor to task collector with label 'simple_task'
    // NOTE: duplicate labels in multiple monitors cause incorrect data aggregation.
    // It is recommended to use unique labels for each monitor and
    // instrument multiple tasks by the `instrument` function.
    task_collector.add("simple_task", monitor.clone());

    // spawn a background task and instrument
    tokio::spawn(monitor.instrument(async {
        loop {
            // do something
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }));

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

And a http server example, you can find in `examples/server.rs`.

## Runtime Metrics

This unstable functionality requires `tokio_unstable`, and the `rt` crate
feature. To enable `tokio_unstable`, the `--cfg` `tokio_unstable` must be passed
to `rustc` when compiling. You can do this by setting the `RUSTFLAGS`
environment variable before compiling your application; e.g.:

```sh
RUSTFLAGS="--cfg tokio_unstable" cargo build
```

Or, by creating the file `.cargo/config.toml` in the root directory of your crate.
If you're using a workspace, put this file in the root directory of your workspace instead.

```toml
[build]
rustflags = ["--cfg", "tokio_unstable"]
rustdocflags = ["--cfg", "tokio_unstable"]
```

- **[`workers_count`]**  
  [`Gauge`] The number of worker threads used by the runtime.
- **[`total_park_count`]**  
  [`Counter] The number of times worker threads parked.
- **[`total_noop_count`]**  
  [`Counter] The number of times worker threads unparked but performed no work before parking again.
- **[`total_steal_count`]**  
  [`Counter] The number of tasks worker threads stole from another worker thread.
- **[`total_steal_operations`]**  
  [`Counter] The number of times worker threads stole tasks from another worker thread.
- **[`num_remote_schedules`]**  
  [`Counter] The number of tasks scheduled from outside of the runtime.
- **[`total_local_schedule_count`]**  
  [`Counter] The number of tasks scheduled from worker threads.
- **[`total_overflow_count`]**  
  [`Counter] The number of times worker threads saturated their local queues.
- **[`total_polls_count`]**  
  [`Counter] The number of tasks that have been polled across all worker threads.
- **[`total_busy_duration`]**  
  [`Counter] The amount of time worker threads were busy.
- **[`injection_queue_depth`]**  
  [`Gauge`] The number of tasks currently scheduled in the runtime's injection queue.
- **[`total_local_queue_depth`]**  
  [`Gauge`] The total number of tasks currently scheduled in workers' local queues.
- **[`elapsed`]**  
  [`Counter`] Total amount of time elapsed since observing runtime metrics.
- **[`budget_forced_yield_count`]**  
  [`Counter`] The number of times that a task was forced to yield because it exhausted its budget.
- **[`io_driver_ready_count`]**  
  [`Counter`] The number of ready events received from the I/O driver.

[`workers_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.workers_count
[`total_park_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_park_count
[`total_noop_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_noop_count
[`total_steal_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_steal_count
[`total_steal_operations`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_steal_operations
[`num_remote_schedules`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.num_remote_schedules
[`total_local_schedule_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_local_schedule_count
[`total_overflow_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_overflow_count
[`total_polls_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_polls_count
[`total_busy_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_busy_duration
[`injection_queue_depth`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.injection_queue_depth
[`total_local_queue_depth`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.total_local_queue_depth
[`elapsed`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.elapsed
[`mean_polls_per_park`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#method.mean_polls_per_park
[`busy_ratio`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#method.busy_ratio
[`budget_forced_yield_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.budget_forced_yield_count
[`io_driver_ready_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.RuntimeMetrics.html#structfield.io_driver_ready_count

## Task Metrics

- **[`instrumented_count`]**  
  [`Gauge`] The number of tasks instrumented in the interval.
- **[`dropped_count`]**  
  [`Gauge] The number of tasks dropped in the interval.
- **[`first_poll_count`]**  
  [`Gauge`] The number of tasks polled for the first time in the interval.
- **[`total_first_poll_delay`]**  
  [`Counter`] The total duration elapsed between the instant tasks are instrumented, and the instant they are first polled.
- **[`total_idled_count`]**  
  [`Counter`] The total number of times that tasks idled, waiting to be awoken.
- **[`total_idle_duration`]**  
  [`Counter`] The total duration that tasks idled.
- **[`total_scheduled_count`]**  
  [`Counter`] The total number of times that tasks were awoken (and then, presumably, scheduled for execution).
- **[`total_scheduled_duration`]**  
  [`Counter`] The total duration that tasks spent waiting to be polled after awakening.
- **[`total_poll_count`]**  
  [`Counter`] The total number of times that tasks were polled.
- **[`total_poll_duration`]**  
  [`Counter`] The total duration elapsed during polls.
- **[`total_fast_poll_count`]**  
  [`Counter`] The total number of times that polling tasks completed swiftly.
- **[`total_fast_poll_duration`]**  
  [`Counter`] The total duration of fast polls.
- **[`total_slow_poll_count`]**  
  [`Counter`] The total number of times that polling tasks completed slowly.
- **[`total_slow_poll_duration`]**  
  [`Counter`] The total duration of slow polls.
- **[`total_short_delay_count`]**  
  [`Counter`] The total count of short scheduling delays.
- **[`total_short_delay_duration`]**  
  [`Counter`] The total duration of short scheduling delays.
- **[`total_long_delay_count`]**  
  [`Counter`] The total count of long scheduling delays.
- **[`total_long_delay_duration`]**  
  [`Counter`] The total duration of long scheduling delays.

[`instrumented_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.instrumented_count
[`dropped_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.dropped_count
[`first_poll_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.first_poll_count
[`total_first_poll_delay`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_first_poll_delay
[`total_idled_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_idled_count
[`total_idle_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_idle_duration
[`total_scheduled_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_scheduled_count
[`total_scheduled_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_scheduled_duration
[`total_poll_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_poll_count
[`total_poll_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_poll_duration
[`total_fast_poll_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_fast_poll_count
[`total_fast_poll_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_fast_poll_duration
[`total_slow_poll_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_slow_poll_count
[`total_slow_poll_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_slow_poll_duration
[`total_short_delay_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_short_delay_count
[`total_short_delay_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_short_delay_duration
[`total_long_delay_count`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_long_delay_count
[`total_long_delay_duration`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#structfield.total_long_delay_duration
[`long_delay_ratio`]: https://docs.rs/tokio-metrics/0.2.*/tokio_metrics/struct.TaskMetrics.html#method.long_delay_ratio

## License

tokio-metrics-collector is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT, and COPYRIGHT for details.
