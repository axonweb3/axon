use crate::metrics::{register_int_gauge_vec, IntGaugeVec};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref MEMORY_TRACE_VEC: IntGaugeVec =
        register_int_gauge_vec!("axon_memory_trace", "axon memory usage status", &[
            "source", "type"
        ])
        .unwrap();
    pub static ref DB_MEMORY_TRACE_VEC: IntGaugeVec =
        register_int_gauge_vec!("axon_db_memory_trace", "axon memory usage status", &[
            "source", "type", "cf"
        ])
        .unwrap();
}
