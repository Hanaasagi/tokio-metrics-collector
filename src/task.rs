use lazy_static::lazy_static;
use parking_lot::RwLock;
use prometheus::{
    core::Desc,
    core::{Collector, Opts},
    proto, CounterVec, IntCounterVec, IntGaugeVec,
};
use std::collections::HashMap;

use tokio_metrics::TaskMetrics as TaskMetricsData;
use tokio_metrics::TaskMonitor;

const TASK_LABEL: &str = "task";

// Reference: https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub instrumented_count: IntGaugeVec,
    pub dropped_count: IntGaugeVec,
    pub first_poll_count: IntGaugeVec,
    pub total_first_poll_delay: CounterVec,
    pub total_idled_count: IntCounterVec,
    pub total_idle_duration: CounterVec,
    pub total_scheduled_count: IntCounterVec,
    pub total_scheduled_duration: CounterVec,
    pub total_poll_count: IntCounterVec,
    pub total_poll_duration: CounterVec,
    pub total_fast_poll_count: IntCounterVec,
    pub total_fast_poll_duration: CounterVec,
    pub total_slow_poll_count: IntCounterVec,
    pub total_slow_poll_duration: CounterVec,
    pub total_short_delay_count: IntCounterVec,
    pub total_long_delay_count: IntCounterVec,
    pub total_short_delay_duration: CounterVec,
    pub total_long_delay_duration: CounterVec,
}

impl TaskMetrics {
    fn new<S: Into<String>>(namespace: S) -> Self {
        let namespace = namespace.into();
        let instrumented_count = IntGaugeVec::new(
            Opts::new(
                "tokio_task_instrumented_count",
                r#"The number of tasks instrumented."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let dropped_count = IntGaugeVec::new(
            Opts::new(
                "tokio_task_dropped_count",
                r#"The number of tasks dropped."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let first_poll_count = IntGaugeVec::new(
            Opts::new(
                "tokio_task_first_poll_count",
                r#"The number of tasks polled for the first time."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_first_poll_delay = CounterVec::new(
            Opts::new(
                "tokio_task_total_first_poll_delay",
                r#"The total duration elapsed between the instant tasks are instrumented, and the instant they are first polled."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL]
        )
        .unwrap();

        let total_idled_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_idled_count",
                r#"The total number of times that tasks idled, waiting to be awoken."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_idle_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_idle_duration",
                r#"The total duration that tasks idled."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_scheduled_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_scheduled_count",
                r#"The total number of times that tasks were awoken (and then, presumably, scheduled for execution)."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL]
        )
        .unwrap();

        let total_scheduled_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_scheduled_duration",
                r#"The total duration that tasks spent waiting to be polled after awakening."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_poll_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_poll_count",
                r#"The total number of times that tasks were polled."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_poll_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_poll_duration",
                r#"The total duration elapsed during polls."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_fast_poll_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_fast_poll_count",
                r#"The amount of time worker threads were busy."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_fast_poll_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_fast_poll_duration",
                r#"The total duration of fast polls."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_slow_poll_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_slow_poll_count",
                r#"The total number of times that polling tasks completed slowly."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_slow_poll_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_slow_poll_duration",
                r#"The total duration of slow polls."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_short_delay_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_short_delay_count",
                r#"The total count of tasks with short scheduling delays."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_long_delay_count = IntCounterVec::new(
            Opts::new(
                "tokio_task_total_long_delay_count",
                r#"The total count of tasks with long scheduling delays."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_short_delay_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_short_delay_duration",
                r#"The total duration of tasks with short scheduling delays."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        let total_long_delay_duration = CounterVec::new(
            Opts::new(
                "tokio_task_total_long_delay_duration",
                r#"The total number of times that a task had a long scheduling duration."#,
            )
            .namespace(namespace.clone()),
            &[TASK_LABEL],
        )
        .unwrap();

        Self {
            instrumented_count,
            dropped_count,
            first_poll_count,
            total_first_poll_delay,
            total_idled_count,
            total_idle_duration,
            total_scheduled_count,
            total_scheduled_duration,
            total_poll_count,
            total_poll_duration,
            total_fast_poll_count,
            total_fast_poll_duration,
            total_slow_poll_count,
            total_slow_poll_duration,
            total_short_delay_count,
            total_long_delay_count,
            total_short_delay_duration,
            total_long_delay_duration,
        }
    }

    fn update(&self, label: &str, data: TaskMetricsData) {
        // counter only support inc and inc_by not set

        macro_rules! update_counter {
            ( $field:ident,  "int" ) => {{
                let past = self.$field.with_label_values(&[label]).get() as u64;
                let new = data.$field as u64;
                debug_assert!(new >= past, "new: {new} >= past: {past}");
                self.$field
                    .with_label_values(&[label])
                    .inc_by(new.saturating_sub(past));
            }};
            ( $field:ident,  "duration" ) => {{
                let past = self.$field.with_label_values(&[label]).get();
                let new = data.$field.as_secs_f64();
                debug_assert!(new >= past, "new: {new} >= past: {past}");
                self.$field.with_label_values(&[label]).inc_by(new - past);
            }};
        }

        update_counter!(total_first_poll_delay, "duration");
        update_counter!(total_idled_count, "int");
        update_counter!(total_idle_duration, "duration");
        update_counter!(total_scheduled_count, "int");
        update_counter!(total_scheduled_duration, "duration");
        update_counter!(total_poll_count, "int");
        update_counter!(total_poll_duration, "duration");
        update_counter!(total_fast_poll_count, "int");
        update_counter!(total_fast_poll_duration, "duration");
        update_counter!(total_slow_poll_count, "int");
        update_counter!(total_slow_poll_duration, "duration");
        update_counter!(total_short_delay_count, "int");
        update_counter!(total_long_delay_count, "int");
        update_counter!(total_short_delay_duration, "duration");
        update_counter!(total_long_delay_duration, "duration");
    }

    fn to_desc(&self) -> Vec<&Desc> {
        let mut desc = vec![];
        desc.extend(self.instrumented_count.desc());
        desc.extend(self.dropped_count.desc());
        desc.extend(self.first_poll_count.desc());
        desc.extend(self.total_first_poll_delay.desc());
        desc.extend(self.total_idled_count.desc());
        desc.extend(self.total_idle_duration.desc());
        desc.extend(self.total_scheduled_count.desc());
        desc.extend(self.total_scheduled_duration.desc());
        desc.extend(self.total_poll_count.desc());
        desc.extend(self.total_poll_duration.desc());
        desc.extend(self.total_fast_poll_count.desc());
        desc.extend(self.total_fast_poll_duration.desc());
        desc.extend(self.total_slow_poll_count.desc());
        desc.extend(self.total_slow_poll_duration.desc());
        desc.extend(self.total_short_delay_count.desc());
        desc.extend(self.total_long_delay_count.desc());
        desc.extend(self.total_short_delay_duration.desc());
        desc.extend(self.total_long_delay_duration.desc());

        assert_eq!(desc.len(), 18);
        desc
    }

    fn to_metrics(&self) -> Vec<proto::MetricFamily> {
        let mut metrics = vec![];
        metrics.extend(self.instrumented_count.collect());
        metrics.extend(self.dropped_count.collect());
        metrics.extend(self.first_poll_count.collect());
        metrics.extend(self.total_first_poll_delay.collect());
        metrics.extend(self.total_idled_count.collect());
        metrics.extend(self.total_idle_duration.collect());
        metrics.extend(self.total_scheduled_count.collect());
        metrics.extend(self.total_scheduled_duration.collect());
        metrics.extend(self.total_poll_count.collect());
        metrics.extend(self.total_poll_duration.collect());
        metrics.extend(self.total_fast_poll_count.collect());
        metrics.extend(self.total_fast_poll_duration.collect());
        metrics.extend(self.total_slow_poll_count.collect());
        metrics.extend(self.total_slow_poll_duration.collect());
        metrics.extend(self.total_short_delay_count.collect());
        metrics.extend(self.total_long_delay_count.collect());
        metrics.extend(self.total_short_delay_duration.collect());
        metrics.extend(self.total_long_delay_duration.collect());

        assert_eq!(metrics.len(), 18);
        metrics
    }
}

#[derive(Debug)]
pub struct TaskCollector {
    metrics: TaskMetrics,
    producer: RwLock<HashMap<String, TaskMonitor>>,
}

impl TaskCollector {
    pub fn new<S: Into<String>>(namespace: S) -> Self {
        let producer = RwLock::new(HashMap::new());
        let metrics = TaskMetrics::new(namespace);

        Self { metrics, producer }
    }

    pub fn add(&self, label: &str, monitor: TaskMonitor) {
        self.producer.write().insert(label.to_string(), monitor);
    }

    pub fn remove(&mut self, label: &str) {
        self.producer.write().remove(label);
    }

    fn get_metrics_data_by_label(&self, label: &str) -> TaskMetricsData {
        let data = self.producer.read().get(label).unwrap().cumulative();
        data
    }
}

impl Collector for TaskCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.metrics.to_desc()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let mut labels = vec![];

        {
            let producer = self.producer.read();

            for (label, _) in producer.iter() {
                labels.push(label.to_string());
            }
        }

        for label in labels {
            let data = self.get_metrics_data_by_label(&label);
            self.metrics.update(&label, data);
        }
        self.metrics.to_metrics()
    }
}

impl Collector for &TaskCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.metrics.to_desc()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let mut labels = vec![];

        {
            let producer = self.producer.read();

            for (label, _) in producer.iter() {
                labels.push(label.to_string());
            }
        }

        for label in labels {
            let data = self.get_metrics_data_by_label(&label);
            self.metrics.update(&label, data);
        }
        self.metrics.to_metrics()
    }
}

lazy_static! {
    static ref DEFAULT_COLLECTOR: TaskCollector = {
        let collector = TaskCollector::new("");

        collector
    };
}

pub fn default_collector() -> &'static TaskCollector {
    lazy_static::initialize(&DEFAULT_COLLECTOR);
    &DEFAULT_COLLECTOR
}
