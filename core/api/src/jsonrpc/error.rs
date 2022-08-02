use jsonrpsee::core::Error;
use jsonrpsee::types::error::{CallError, ErrorObject};

use protocol::codec::hex_encode;
use protocol::types::{ExitReason, TxResp};

use core_executor::decode_revert_msg;

const EXEC_ERROR: i32 = -32015;

#[derive(Clone, Debug)]
pub enum RpcError {
    VM(TxResp),
}

impl From<RpcError> for Error {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::VM(resp) => vm_err(resp),
        }
    }
}

pub fn vm_err(resp: TxResp) -> Error {
    let data = match resp.exit_reason {
        ExitReason::Revert(_) => {
            if resp.ret.is_empty() {
                None
            } else {
                Some(format!("0x{}", hex_encode(&resp.ret)))
            }
        }
        ExitReason::Error(err) => Some(format!("{:?}", err)),
        ExitReason::Fatal(fatal) => Some(format!("{:?}", fatal)),
        _ => None,
    };

    into_rpc_err(ErrorObject::owned(
        EXEC_ERROR,
        decode_revert_msg(&resp.ret),
        data,
    ))
}

fn into_rpc_err(obj: ErrorObject<'static>) -> Error {
    CallError::Custom(obj).into()
}
