use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VMResp {
    pub exit_code: i8,
    pub cycles:    u64,
}
