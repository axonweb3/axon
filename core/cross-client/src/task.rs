use reqwest::Url;

use protocol::types::{Requests, ValidatorExtend, H160};

pub struct RequestCkbTask {
    address:     H160,
    validators:  Vec<ValidatorExtend>,
    mercury_uri: Url,
    ckb_uri:     Url,
    requests:    Requests,
}
