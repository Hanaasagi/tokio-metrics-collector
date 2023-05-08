use lazy_static::lazy_static;

use prometheus::{
    core::Desc,
    core::{Collector, Opts},
    proto, Counter, IntCounter, IntGauge,
};
use std::sync::Mutex;
use tokio;

use tokio_metrics::RuntimeIntervals;
use tokio_metrics::RuntimeMetrics as RuntimeMetricsData;
use tokio_metrics::RuntimeMonitor;

const METRICS_COUNT: usize = 15;

// Reference: https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html
#[derive(Debug)]
struct RuntimeMetrics {
    workers_count: IntGauge,
    total_park_count: IntCounter,
    // pub max_park_count: u64,
    // pub min_park_count: u64,
    total_noop_count: IntCounter,
    // pub max_noop_count: u64,
    // pub min_noop_count: u64,
    total_steal_count: IntCounter,
    // pub max_steal_count: u64,
    // pub min_steal_count: u64,
    total_steal_operations: IntCounter,
    // pub max_steal_operations: u64,
    // pub min_steal_operations: u64,
    num_remote_schedules: IntCounter,
    total_local_schedule_count: IntCounter,
    // pub max_local_schedule_count: u64,
    // pub min_local_schedule_count: u64,
    total_overflow_count: IntCounter,
    // pub max_overflow_count: u64,
    // pub min_overflow_count: u64,
    total_polls_count: IntCounter,
    // pub max_polls_count: u64,
    // pub min_polls_count: u64,
    total_busy_duration: Counter,
    // pub max_busy_duration: Duration,
    // pub min_busy_duration: Duration,
    injection_queue_depth: IntGauge,
    total_local_queue_depth: IntGauge,
    // pub max_local_queue_depth: usize,
    // pub min_local_queue_depth: usize,
    elapsed: Counter,
    budget_forced_yield_count: IntCounter,
    io_driver_ready_count: IntCounter,
}

impl RuntimeMetrics {
    fn new<S: Into<String>>(namespace: S) -> Self {
        let namespace = namespace.into();
        let workers_count = IntGauge::with_opts(
            Opts::new(
                "tokio_workers_count",
                r#"The number of worker threads used by the runtime."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_park_count = IntCounter::with_opts(
            Opts::new(
                "tokio_total_park_count",
                r#"The number of times worker threads parked."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_noop_count = IntCounter::with_opts(
            Opts::new(
                "tokio_total_noop_count",
                r#"The number of times worker threads unparked but performed no work before parking again."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_steal_count = IntCounter::with_opts(
            Opts::new(
                "tokio_total_steal_count",
                r#"The number of tasks worker threads stole from another worker thread."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_steal_operations = IntCounter::with_opts(
            Opts::new(
                "tokio_total_seal_operations",
                r#"The number of times worker threads stole tasks from another worker thread."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let num_remote_schedules = IntCounter::with_opts(
            Opts::new(
                "tokio_num_remote_schedules",
                r#"The number of tasks scheduled from outside of the runtime."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_local_schedule_count = IntCounter::with_opts(
            Opts::new(
                "tokio_total_local_schedule_count",
                r#"The number of tasks scheduled from worker threads."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_overflow_count = IntCounter::with_opts(
            Opts::new(
                "tokio_total_overflow_count",
                r#"The number of times worker threads saturated their local queues."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_polls_count = IntCounter::with_opts(
            Opts::new(
                "tokio_total_polls_count",
                r#"The number of tasks that have been polled across all worker threads."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_busy_duration = Counter::with_opts(
            Opts::new(
                "tokio_total_busy_duration",
                r#"The amount of time worker threads were busy."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let injection_queue_depth = IntGauge::with_opts(
            Opts::new(
                "tokio_injection_queue_depth",
                r#"The number of tasks currently scheduled in the runtime’s injection queue."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let total_local_queue_depth = IntGauge::with_opts(
            Opts::new(
                "tokio_total_local_queue_depth",
                r#"The total number of tasks currently scheduled in workers’ local queues."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let elapsed = Counter::with_opts(
            Opts::new(
                "tokio_elapsed",
                r#"Total amount of time elapsed since observing runtime metrics."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        let budget_forced_yield_count = IntCounter::with_opts(
            Opts::new(
                "tokio_budget_forced_yield_count",
                r#"Returns the number of times that tasks have been forced to yield back to the scheduler after exhausting their task budgets."#
            )
                .namespace(namespace.clone()),
        )
        .unwrap();

        let io_driver_ready_count = IntCounter::with_opts(
            Opts::new(
                "tokio_io_driver_ready_count",
                r#"Returns the number of ready events processed by the runtime’s I/O driver."#,
            )
            .namespace(namespace.clone()),
        )
        .unwrap();

        Self {
            workers_count,
            total_park_count,
            total_noop_count,
            total_steal_count,
            total_steal_operations,
            num_remote_schedules,
            total_local_schedule_count,
            total_overflow_count,
            total_polls_count,
            total_busy_duration,
            injection_queue_depth,
            total_local_queue_depth,
            elapsed,
            budget_forced_yield_count,
            io_driver_ready_count,
        }
    }

    fn update(&self, data: RuntimeMetricsData) {
        // counter only support inc and inc_by not set
        macro_rules! update_counter {
            ( $field:ident, "int" ) => {{
                // let prev = self.$field.get() as u64;
                let new = data.$field as u64;
                // debug_assert!(new >= prev, "new: {new} >= prev: {prev}");
                // self.$field.inc_by(new.saturating_sub(prev));
                self.$field.inc_by(new);
            }};
            ( $field:ident, "duration" ) => {{
                // let prev = self.$field.get();
                let new = data.$field.as_secs_f64();
                // debug_assert!(new >= prev, "new: {new} >= prev: {prev}");
                // self.$field.inc_by(new - prev);
                self.$field.inc_by(new);
            }};
        }

        self.workers_count.set(data.workers_count as i64);

        update_counter!(total_park_count, "int");
        update_counter!(total_noop_count, "int");
        update_counter!(total_steal_count, "int");
        update_counter!(total_steal_operations, "int");
        update_counter!(num_remote_schedules, "int");
        update_counter!(total_local_schedule_count, "int");
        update_counter!(total_overflow_count, "int");
        update_counter!(total_polls_count, "int");
        update_counter!(total_busy_duration, "duration");

        self.injection_queue_depth
            .set(data.injection_queue_depth as i64);

        self.total_local_queue_depth
            .set(data.total_local_queue_depth as i64);
        update_counter!(elapsed, "duration");
        update_counter!(budget_forced_yield_count, "int");
        update_counter!(io_driver_ready_count, "int");
    }

    fn to_desc(&self) -> Vec<&Desc> {
        let mut desc = vec![];
        desc.extend(self.workers_count.desc());
        desc.extend(self.total_park_count.desc());
        desc.extend(self.total_noop_count.desc());
        desc.extend(self.total_steal_count.desc());
        desc.extend(self.total_steal_operations.desc());
        desc.extend(self.num_remote_schedules.desc());
        desc.extend(self.total_local_schedule_count.desc());
        desc.extend(self.total_overflow_count.desc());
        desc.extend(self.total_polls_count.desc());
        desc.extend(self.total_busy_duration.desc());
        desc.extend(self.injection_queue_depth.desc());
        desc.extend(self.total_local_queue_depth.desc());
        desc.extend(self.elapsed.desc());
        desc.extend(self.budget_forced_yield_count.desc());
        desc.extend(self.io_driver_ready_count.desc());

        debug_assert_eq!(desc.len(), METRICS_COUNT);

        desc
    }

    fn to_metrics(&self) -> Vec<proto::MetricFamily> {
        let mut metrics = vec![];
        metrics.extend(self.workers_count.collect());
        metrics.extend(self.total_park_count.collect());
        metrics.extend(self.total_noop_count.collect());
        metrics.extend(self.total_steal_count.collect());
        metrics.extend(self.total_steal_operations.collect());
        metrics.extend(self.num_remote_schedules.collect());
        metrics.extend(self.total_local_schedule_count.collect());
        metrics.extend(self.total_overflow_count.collect());
        metrics.extend(self.total_polls_count.collect());
        metrics.extend(self.total_busy_duration.collect());
        metrics.extend(self.injection_queue_depth.collect());
        metrics.extend(self.total_local_queue_depth.collect());
        metrics.extend(self.elapsed.collect());
        metrics.extend(self.budget_forced_yield_count.collect());
        metrics.extend(self.io_driver_ready_count.collect());

        debug_assert_eq!(metrics.len(), METRICS_COUNT);

        metrics
    }
}

/// RuntimeCollector
pub struct RuntimeCollector {
    metrics: RuntimeMetrics,
    producer: Mutex<RuntimeIntervals>,
}

impl RuntimeCollector {
    /// Create a [`RuntimeCollector`] in namespace.
    pub fn new<S: Into<String>>(monitor: RuntimeMonitor, namespace: S) -> Self {
        let producer = Mutex::new(monitor.intervals());
        let metrics = RuntimeMetrics::new(namespace);
        Self { metrics, producer }
    }

    fn get_metrics_data(&self) -> RuntimeMetricsData {
        // unwrap is safe here, because the iterator is infinite.
        let data = self.producer.lock().unwrap().next().unwrap();

        data
    }
}

impl Default for RuntimeCollector {
    fn default() -> Self {
        let handle = tokio::runtime::Handle::current();
        // construct the runtime metrics monitor
        let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&handle);
        Self::new(runtime_monitor, "")
    }
}

impl Collector for RuntimeCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.metrics.to_desc()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let data = self.get_metrics_data();
        self.metrics.update(data);

        self.metrics.to_metrics()
    }
}

impl Collector for &RuntimeCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.metrics.to_desc()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let data = self.get_metrics_data();
        self.metrics.update(data);

        self.metrics.to_metrics()
    }
}

lazy_static! {
    static ref DEFAULT_COLLECTOR: RuntimeCollector = {
        let collector = RuntimeCollector::default();

        collector
    };
}

/// Get the global [`RuntimeCollector`], the namespace is under "".
pub fn default_collector() -> &'static RuntimeCollector {
    lazy_static::initialize(&DEFAULT_COLLECTOR);
    &DEFAULT_COLLECTOR
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime;

    #[test]
    fn test_runtime_collector_descs() {
        let rt = runtime::Builder::new_current_thread().build().unwrap();
        let mt = tokio_metrics::RuntimeMonitor::new(&rt.handle());
        let rc = RuntimeCollector::new(mt, "");

        let descs = rc.desc();
        assert_eq!(descs.len(), METRICS_COUNT);
        assert_eq!(descs[0].fq_name, "tokio_workers_count".to_string());
        assert_eq!(
            descs[0].help,
            "The number of worker threads used by the runtime."
        );
        assert_eq!(descs[0].variable_labels.len(), 0);
    }

    #[test]
    fn test_runtime_collector_metrics() {
        let rt = runtime::Builder::new_current_thread().build().unwrap();
        let mt = tokio_metrics::RuntimeMonitor::new(&rt.handle());
        let rc = RuntimeCollector::new(mt, "");

        let metrics = rc.collect();
        assert_eq!(metrics.len(), METRICS_COUNT);
        assert_eq!(metrics[0].get_name(), "tokio_workers_count");
        assert_eq!(
            metrics[0].get_help(),
            "The number of worker threads used by the runtime."
        );
        assert_eq!(metrics[0].get_metric().len(), 1);
        assert_eq!(metrics[0].get_metric()[0].get_gauge().get_value(), 1.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_default() {
        let collector = default_collector();
        assert_eq!(collector.desc().len(), METRICS_COUNT);
        let metrics = collector.collect();
        assert_eq!(metrics.len(), METRICS_COUNT);
        assert_eq!(metrics[0].get_name(), "tokio_workers_count");
        assert_eq!(
            metrics[0].get_help(),
            "The number of worker threads used by the runtime."
        );
        assert_eq!(metrics[0].get_metric().len(), 1);
        // 8 worker threads
        assert_eq!(metrics[0].get_metric()[0].get_gauge().get_value(), 8.0);
    }

    #[tokio::test]
    async fn test_integrated_with_prometheus() {
        use prometheus::Encoder;

        let tc = RuntimeCollector::default();

        prometheus::default_registry()
            .register(Box::new(tc))
            .unwrap();

        let encoder = prometheus::TextEncoder::new();

        let mut buffer = Vec::new();
        encoder
            .encode(&prometheus::default_registry().gather(), &mut buffer)
            .expect("Failed to encode");
        String::from_utf8(buffer.clone()).expect("Failed to convert to string.");
    }

    #[test]
    fn test_send() {
        fn test<C: Send>() {}
        test::<DEFAULT_COLLECTOR>();
    }

    #[test]
    fn test_sync() {
        fn test<C: Sync>() {}
        test::<DEFAULT_COLLECTOR>();
    }
}
