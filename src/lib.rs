#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

mod task;

pub use task::default_collector as default_task_collector;
pub use task::TaskCollector;

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
