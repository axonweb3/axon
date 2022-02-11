use std::io::Error;

use metrics_derive::metrics_rpc;

pub struct ExampleRpc;

impl ExampleRpc {
    // The expanded code
    //
    // fn send_raw_transaction(&self, input: u64) -> Result<String, Error> {
    //     let inst = std::time::Instant::now();
    //     let ret = { Ok(input.to_string()) };
    //     if ret.is_err() {
    //         common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
    //             .eth_sendRawTransaction
    //             .failure
    //             .inc();
    //         return ret;
    //     }
    //     common_apm::metrics::api::API_REQUEST_RESULT_COUNTER_VEC_STATIC
    //         .eth_sendRawTransaction
    //         .success
    //         .inc();
    //     common_apm::metrics::api::API_REQUEST_TIME_HISTOGRAM_STATIC
    //         .eth_sendRawTransaction
    //         .observe(common_apm::metrics::duration_to_sec(inst.elapsed()));
    //     ret
    // }
    #[metrics_rpc("eth_sendRawTransaction")]
    fn send_raw_transaction(&self, input: u64) -> Result<String, Error> {
        Ok(input.to_string())
    }
}

fn main() {}
