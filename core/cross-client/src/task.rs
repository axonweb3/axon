use protocol::types::{Requests, ValidatorExtend, H160};

pub struct RequestCkbTask {
    address:    H160,
    validators: Vec<ValidatorExtend>,
    requests:   Requests,
}

impl RequestCkbTask {
    pub fn new(address: H160, validators: Vec<ValidatorExtend>, requests: Requests) -> Self {
        RequestCkbTask {
            address,
            validators,
            requests,
        }
    }
}
