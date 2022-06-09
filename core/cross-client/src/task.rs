use reqwest::Url;

use protocol::types::{ValidatorExtend, H160, Requests};

pub struct RequestCkbTask {
    address:     H160,
    validators:  Vec<ValidatorExtend>,
    mercury_uri: Url,
    ckb_uri:     Url,
    requests:    Requests,
}
