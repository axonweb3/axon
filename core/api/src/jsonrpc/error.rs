use jsonrpsee::core::Error;
use jsonrpsee::types::error::{CallError, ErrorObject};

use protocol::codec::hex_encode;
use protocol::types::{ExitReason, TxResp};

const EXEC_ERROR: i32 = -32015;

pub fn vm_err(resp: TxResp) -> Error {
    let data = match resp.exit_reason {
        ExitReason::Revert(_) => format!("0x{}", hex_encode(&resp.ret)),
        ExitReason::Error(err) => format!("{:?}", err),
        ExitReason::Fatal(fatal) => format!("{:?}", fatal),
        _ => unreachable!(),
    };

    into_rpc_err(ErrorObject::owned(
        EXEC_ERROR,
        "VM execution error.",
        Some(data),
    ))
}

fn into_rpc_err(obj: ErrorObject<'static>) -> Error {
    CallError::Custom(obj).into()
}
