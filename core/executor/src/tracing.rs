use std::{cell::RefCell, ptr::NonNull, rc::Rc};

use evm::gasometer::tracing as gas_tracing;
use evm::{tracing as evm_tracing, Capture};
use evm_runtime::tracing as runtime_tracing;

use protocol::types::{Hex, TransactionTrace as Web3TransactionTrace, H160, H256, U256};

use crate::tracing::{wrapped_event::WrappedEvent, wrapped_opcode::Opcode};

macro_rules! trace_type {
    ($name: ident, $type_: ty) => {
        #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub $type_);
    };

    ($name: ident, Vec, $type_: ty) => {
        #[derive(Default, Clone, Debug, PartialEq, Eq)]
        pub struct $name(pub Vec<$type_>);

        impl $name {
            #[allow(dead_code)]
            pub fn len(&self) -> usize {
                self.0.len()
            }
        }
    };

    ($name: ident, BTreeMap, ($key: ident, $val: ident)) => {
        #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub std::collections::BTreeMap<$key, $val>);

        impl $name {
            pub fn insert(&mut self, key: $key, value: $val) {
                self.0.insert(key, value);
            }
        }
    };
}

pub fn trace_using<T, R, F>(listener: &mut T, enable_gasometer: bool, f: F) -> R
where
    T: evm_tracing::EventListener
        + runtime_tracing::EventListener
        + gas_tracing::EventListener
        + 'static,
    F: FnOnce() -> R,
{
    let mut evm_listener = SharedMutableReference::new(listener);
    let mut runtime_listener = evm_listener.clone();

    if enable_gasometer {
        let mut gas_listener = evm_listener.clone();

        return evm_tracing::using(&mut gas_listener, || {
            runtime_tracing::using(&mut runtime_listener, || {
                gas_tracing::using(&mut evm_listener, f)
            })
        });
    }

    evm_tracing::using(&mut runtime_listener, || {
        runtime_tracing::using(&mut evm_listener, f)
    })
}

struct SharedMutableReference<T> {
    pointer: Rc<RefCell<NonNull<T>>>,
}

impl<T> SharedMutableReference<T> {
    fn new(reference: &mut T) -> Self {
        let ptr = NonNull::new(reference as _).unwrap();
        Self {
            pointer: Rc::new(RefCell::new(ptr)),
        }
    }

    fn clone(&self) -> Self {
        Self {
            pointer: Rc::clone(&self.pointer),
        }
    }
}

impl<T: evm_tracing::EventListener> evm_tracing::EventListener for SharedMutableReference<T> {
    fn event(&mut self, event: evm_tracing::Event) {
        unsafe {
            self.pointer.borrow_mut().as_mut().event(event);
        }
    }
}

impl<T: runtime_tracing::EventListener> runtime_tracing::EventListener
    for SharedMutableReference<T>
{
    fn event(&mut self, event: runtime_tracing::Event) {
        unsafe {
            self.pointer.borrow_mut().as_mut().event(event);
        }
    }
}

impl<T: gas_tracing::EventListener> gas_tracing::EventListener for SharedMutableReference<T> {
    fn event(&mut self, event: gas_tracing::Event) {
        unsafe {
            self.pointer.borrow_mut().as_mut().event(event);
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AxonListener {
    logs:               Vec<TraceLog>,
    type_:              Option<String>,
    current:            TraceLog,
    current_memory_gas: u64,
    gas_used:           u64,
    failed:             bool,
    output:             Vec<u8>,
}

impl evm_tracing::EventListener for AxonListener {
    fn event(&mut self, event: evm_tracing::Event) {
        use evm::tracing::Event;

        #[cfg(feature = "tracing")]
        log::info!("EVM event {:?}", WrappedEvent::from(&event));

        match event {
            Event::TransactCall {
                caller,
                address,
                value,
                data,
                ..
            } => {
                self.current.from = caller;
                self.current.to = address;
                self.current.value = value;
                self.current.input = data.to_vec();
                self.current.depth += 1;

                if self.type_.is_none() {
                    self.type_ = Some("CALL".to_string());
                }
            }

            Event::TransactCreate {
                caller,
                value,
                init_code,
                address,
                ..
            } => {
                self.current.from = caller;
                self.current.to = address;
                self.current.value = value;
                self.current.input = init_code.to_vec();
                self.current.depth += 1;

                if self.type_.is_none() {
                    self.type_ = Some("CREATE".to_string());
                }
            }

            Event::TransactCreate2 {
                caller,
                value,
                init_code,
                address,
                ..
            } => {
                self.current.from = caller;
                self.current.to = address;
                self.current.value = value;
                self.current.input = init_code.to_vec();
                self.current.depth += 1;

                if self.type_.is_none() {
                    self.type_ = Some("CREATE2".to_string());
                }
            }

            Event::Exit { return_value, .. } => {
                // If the depth is not zero then an error must have occurred to exit early.
                if self.current.depth != 0 {
                    self.failed = true;
                    self.output = return_value.to_vec();
                }
            }
            // Others contain no useful information
            _ => (),
        }
    }
}

impl runtime_tracing::EventListener for AxonListener {
    fn event(&mut self, event: runtime_tracing::Event) {
        use evm_runtime::tracing::Event;

        #[cfg(feature = "tracing")]
        log::info!("EVM runtime event {:?}", WrappedEvent::from(&event));

        match event {
            Event::Step {
                context,
                opcode,
                position,
                stack,
                memory,
            } => {
                if let Ok(pc) = position {
                    self.current.program_counter = *pc;
                }

                self.current.type_ = Opcode::from(opcode).to_string();
                self.current.from = context.caller;
                self.current.to = context.address;
                self.current.opcode = opcode.into();
                self.current.stack = stack.data().as_slice().into();
                self.current.memory = memory.data().as_slice().into();
            }

            Event::StepResult {
                result,
                return_value,
            } => {
                match result {
                    Ok(_) => {
                        // Step completed, push current log into the record
                        self.logs.push(self.current.clone());
                    }
                    Err(Capture::Exit(reason)) => {
                        // Step completed, push current log into the record, reduce depth by 1
                        self.logs.push(self.current.clone());
                        self.current.output = return_value.to_vec();
                        self.current.depth = self.current.depth.saturating_sub(1);

                        // if the depth is 0 then the transaction is complete
                        if self.current.depth == 0 {
                            if !return_value.is_empty() {
                                self.output = return_value.to_vec();
                            }

                            if !reason.is_succeed() {
                                self.failed = true;
                            }
                        }
                    }

                    Err(Capture::Trap(opcode)) => {
                        // "Trap" here means that there is some opcode which has special
                        // handling logic outside the core `step` function. This means the
                        // `StepResult` does not necessarily indicate the current log
                        // is finished yet. In particular, `SLoad` and `SStore` events come
                        // _after_ the `StepResult`, but still correspond to the current step.
                        if opcode == &evm::Opcode::SLOAD || opcode == &evm::Opcode::SSTORE {
                            self.logs.push(self.current.clone());
                        }
                    }
                }
            }

            Event::SLoad { index, value, .. } => {
                self.current
                    .storage
                    .insert(LogStorageKey(index.0), LogStorageValue(value.0));
                self.logs.push(self.current.clone());
            }

            Event::SStore { index, value, .. } => {
                self.current
                    .storage
                    .insert(LogStorageKey(index.0), LogStorageValue(value.0));
                self.logs.push(self.current.clone());
            }
        }
    }
}

impl gas_tracing::EventListener for AxonListener {
    fn event(&mut self, event: gas_tracing::Event) {
        use gas_tracing::Event;

        #[cfg(feature = "tracing")]
        log::info!("EVM gas event {:?}", WrappedEvent::from(&event));

        match event {
            Event::RecordCost { cost, .. } => {
                self.current.gas_cost = cost;
            }

            Event::RecordDynamicCost {
                gas_cost,
                memory_gas,
                ..
            } => {
                // In SputnikVM memory gas is cumulative (ie this event always shows the total)
                // gas spent on memory up to this point. But geth traces simply show how much
                // gas each step took, regardless of how that gas was used. So if this step
                // caused an increase to the memory gas then we need to record that.
                let memory_cost_diff = if memory_gas > self.current_memory_gas {
                    memory_gas - self.current_memory_gas
                } else {
                    0
                };
                self.current_memory_gas = memory_gas;
                self.current.gas_cost = gas_cost + memory_cost_diff;
            }

            Event::RecordRefund { snapshot, .. } => {
                // This one seems to show up at the end of a transaction, so it can be used to
                // set the total gas used.
                if let Some(snapshot) = snapshot {
                    self.gas_used = snapshot.used_gas;
                }
            }

            Event::RecordTransaction { .. } | Event::RecordStipend { .. } => (),
        }
    }
}

impl AxonListener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> TransactionTrace {
        TransactionTrace {
            gas:          self.gas_used,
            failed:       self.failed,
            return_value: self.output,
            struct_logs:  Logs(self.logs),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TraceLog {
    pub type_:           String,
    pub depth:           usize,
    pub from:            H160,
    pub to:              H160,
    pub value:           U256,
    pub input:           Vec<u8>,
    pub output:          Vec<u8>,
    pub error:           Option<String>,
    pub gas:             u64,
    pub gas_cost:        u64,
    pub opcode:          Opcode,
    pub program_counter: usize,
    pub memory:          LogMemory,
    pub stack:           LogStack,
    pub storage:         LogStorage,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TransactionTrace {
    gas:          u64,
    failed:       bool,
    return_value: Vec<u8>,
    struct_logs:  Logs,
}

impl TryFrom<TransactionTrace> for Web3TransactionTrace {
    type Error = String;

    fn try_from(mut value: TransactionTrace) -> Result<Self, Self::Error> {
        let log_len = value.struct_logs.len();

        if log_len == 0 {
            return Err("No logs".to_string());
        }

        let root = value.struct_logs.0.swap_remove(0);
        let mut ret = Web3TransactionTrace {
            type_:         root.type_,
            from:          root.from,
            to:            root.to,
            value:         root.value,
            input:         Hex::encode(root.input),
            output:        Hex::encode(&value.return_value),
            gas:           U256::zero(),
            gas_used:      value.gas.into(),
            error:         None,
            revert_reason: None,
            calls:         None,
        };

        if log_len > 1 {
            let logs = &value.struct_logs.0;

            if !logs.windows(2).all(|pair| {
                let (prev, next) = (&pair[0], &pair[1]);

                prev.depth == next.depth
                    || prev.depth + 1 == next.depth
                    || prev.depth == next.depth + 1
            }) {
                return Err("Log depth variation is not smooth".to_string());
            }

            ret.calls = Some(build_subcalls(logs, &mut 0, logs[0].depth));
        }

        Ok(ret)
    }
}

fn build_subcalls(
    traces: &[TraceLog],
    index: &mut usize,
    current_depth: usize,
) -> Vec<Web3TransactionTrace> {
    let trace_len = traces.len();
    let mut stack = Vec::new();

    while *index < trace_len {
        let t_log = &traces[*index];

        if t_log.depth == current_depth {
            stack.push(Web3TransactionTrace {
                type_:         t_log.type_.clone(),
                from:          t_log.from,
                to:            t_log.to,
                value:         t_log.value,
                gas:           t_log.gas.into(),
                gas_used:      t_log.gas_cost.into(),
                input:         Hex::encode(&t_log.input),
                output:        Hex::encode(&t_log.output),
                error:         None,
                revert_reason: None,
                calls:         None,
            });

            *index += 1;
        } else if t_log.depth == (current_depth + 1) {
            stack.last_mut().unwrap().calls =
                Some(build_subcalls(traces, index, current_depth + 1));
        } else {
            return stack;
        }
    }

    stack
}

trace_type!(LogStorageKey, [u8; 32]);
trace_type!(LogStorageValue, [u8; 32]);
trace_type!(LogMemory, Vec, [u8; 32]);
trace_type!(LogStack, Vec, [u8; 32]);
trace_type!(Logs, Vec, TraceLog);
trace_type!(LogStorage, BTreeMap, (LogStorageKey, LogStorageValue));

impl From<&[u8]> for LogMemory {
    fn from(bytes: &[u8]) -> Self {
        let mut result = Vec::with_capacity(bytes.len() / 32);
        let mut buf = [0u8; 32];
        for (i, b) in bytes.iter().enumerate() {
            let j = i % 32;
            buf[j] = *b;
            if j == 31 {
                result.push(buf)
            }
        }
        Self(result)
    }
}

impl From<&[H256]> for LogStack {
    fn from(stack: &[H256]) -> Self {
        let vec = stack.iter().map(|bytes| bytes.0).collect();
        Self(vec)
    }
}

mod wrapped_opcode {
    use std::cmp::{Eq, PartialEq};
    use std::fmt::{Display, Formatter, Result};

    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Opcode(u8);

    impl From<evm::Opcode> for Opcode {
        fn from(value: evm::Opcode) -> Self {
            Opcode::new(value.as_u8())
        }
    }

    impl Display for Opcode {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            Display::fmt(
                &revm_interpreter::opcode::OpCode::try_from_u8(self.0)
                    .unwrap()
                    .as_str(),
                f,
            )
        }
    }

    impl Opcode {
        pub fn new(value: u8) -> Self {
            Self(value)
        }
    }
}

#[cfg(feature = "tracing")]
mod wrapped_event {
    use super::*;
    use std::fmt::{Debug, Formatter, Result};

    #[derive(Clone)]
    pub enum WrappedEvent<'a> {
        Evm(&'a evm_tracing::Event<'a>),
        Runtime(&'a runtime_tracing::Event<'a>),
        Gas(&'a gas_tracing::Event),
    }

    impl<'a> Debug for WrappedEvent<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                WrappedEvent::Evm(event) => {
                    use evm_tracing::Event;
                    match event {
                        Event::Create {
                            caller,
                            address,
                            scheme,
                            value,
                            target_gas,
                            ..
                        } => f
                            .debug_struct("Create")
                            .field("caller", caller)
                            .field("address", address)
                            .field("scheme", scheme)
                            .field("value", value)
                            .field("gas", target_gas)
                            .finish_non_exhaustive(),
                        Event::TransactCreate {
                            caller,
                            value,
                            gas_limit,
                            address,
                            ..
                        } => f
                            .debug_struct("Create")
                            .field("caller", caller)
                            .field("value", value)
                            .field("gas", gas_limit)
                            .field("address", address)
                            .finish_non_exhaustive(),
                        Event::TransactCreate2 {
                            caller,
                            value,
                            salt,
                            gas_limit,
                            address,
                            ..
                        } => f
                            .debug_struct("Create2")
                            .field("caller", caller)
                            .field("value", value)
                            .field("salt", salt)
                            .field("gas", gas_limit)
                            .field("address", address)
                            .finish_non_exhaustive(),
                        _ => Debug::fmt(event, f),
                    }
                }
                WrappedEvent::Runtime(event) => {
                    use runtime_tracing::Event;
                    match event {
                        Event::Step {
                            context,
                            opcode,
                            position,
                            stack,
                            ..
                        } => f
                            .debug_struct("Step")
                            .field("context", context)
                            .field("opcode", opcode)
                            .field("position", position)
                            .field("stack", stack)
                            .finish_non_exhaustive(),
                        Event::StepResult {
                            result,
                            return_value,
                        } => f
                            .debug_struct("StepResult")
                            .field("result", result)
                            .field("return_len", &return_value.len())
                            .field("return_value", return_value)
                            .finish(),
                        _ => Debug::fmt(event, f),
                    }
                }
                WrappedEvent::Gas(event) => Debug::fmt(event, f),
            }
        }
    }

    impl<'a> From<&'a evm_tracing::Event<'a>> for WrappedEvent<'a> {
        fn from(event: &'a evm_tracing::Event) -> Self {
            Self::Evm(event)
        }
    }

    impl<'a> From<&'a runtime_tracing::Event<'a>> for WrappedEvent<'a> {
        fn from(event: &'a runtime_tracing::Event) -> Self {
            Self::Runtime(event)
        }
    }

    impl<'a> From<&'a gas_tracing::Event> for WrappedEvent<'a> {
        fn from(event: &'a gas_tracing::Event) -> Self {
            Self::Gas(event)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_trace(depth: usize) -> TraceLog {
        TraceLog {
            depth,
            value: depth.into(),
            ..Default::default()
        }
    }

    #[test]
    fn test_build_subcalls() {
        let logs = vec![
            mock_trace(1),
            mock_trace(1),
            mock_trace(2),
            mock_trace(3),
            mock_trace(2),
            mock_trace(1),
            mock_trace(2),
            mock_trace(1),
        ];

        let res = build_subcalls(&logs, &mut 0, 1);
        let data = r#"[
        {
            "type_": "",
            "from": "0x0000000000000000000000000000000000000000",
            "to": "0x0000000000000000000000000000000000000000",
            "value": "0x1",
            "gas": "0x0",
            "gas_used": "0x0",
            "input": "0x",
            "output": "0x"
        },
        {
            "type_": "",
            "from": "0x0000000000000000000000000000000000000000",
            "to": "0x0000000000000000000000000000000000000000",
            "value": "0x1",
            "gas": "0x0",
            "gas_used": "0x0",
            "input": "0x",
            "output": "0x",
            "calls": [
                {
                    "type_": "",
                    "from": "0x0000000000000000000000000000000000000000",
                    "to": "0x0000000000000000000000000000000000000000",
                    "value": "0x2",
                    "gas": "0x0",
                    "gas_used": "0x0",
                    "input": "0x",
                    "output": "0x",
                    "calls": [
                        {
                            "type_": "",
                            "from": "0x0000000000000000000000000000000000000000",
                            "to": "0x0000000000000000000000000000000000000000",
                            "value": "0x3",
                            "gas": "0x0",
                            "gas_used": "0x0",
                            "input": "0x",
                            "output": "0x"
                        }
                    ]
                },
                {
                    "type_": "",
                    "from": "0x0000000000000000000000000000000000000000",
                    "to": "0x0000000000000000000000000000000000000000",
                    "value": "0x2",
                    "gas": "0x0",
                    "gas_used": "0x0",
                    "input": "0x",
                    "output": "0x"
                }
            ]
        },
        {
            "type_": "",
            "from": "0x0000000000000000000000000000000000000000",
            "to": "0x0000000000000000000000000000000000000000",
            "value": "0x1",
            "gas": "0x0",
            "gas_used": "0x0",
            "input": "0x",
            "output": "0x",
            "calls": [
                {
                    "type_": "",
                    "from": "0x0000000000000000000000000000000000000000",
                    "to": "0x0000000000000000000000000000000000000000",
                    "value": "0x2",
                    "gas": "0x0",
                    "gas_used": "0x0",
                    "input": "0x",
                    "output": "0x"
                }
            ]
        },
        {
            "type_": "",
            "from": "0x0000000000000000000000000000000000000000",
            "to": "0x0000000000000000000000000000000000000000",
            "value": "0x1",
            "gas": "0x0",
            "gas_used": "0x0",
            "input": "0x",
            "output": "0x"
        }]"#;

        assert_eq!(
            res,
            serde_json::from_str::<Vec<Web3TransactionTrace>>(data).unwrap()
        );
    }
}
