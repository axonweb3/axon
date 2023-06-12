use std::{cell::RefCell, ops::Index, ptr::NonNull, rc::Rc};

use evm::gasometer::tracing as gas_tracing;
use evm::{tracing as evm_tracing, Capture, Opcode};
use evm_runtime::tracing as runtime_tracing;

use protocol::types::H256;

use crate::tracing::wrapped_event::WrappedEvent;

macro_rules! trace_type {
    ($($name: ident,)*) => {
        $(
            #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $name(pub [u8; 32]);

            impl $name {
                pub fn into_raw(self) -> [u8; 32] {
                    self.0
                }
            }
        )*
    };

    (Vec $(, $name: ident)*) => {
        $(
            #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $name(pub Vec<[u8; 32]>);

            impl $name {
                pub fn len(&self) -> usize {
                    self.0.len()
                }

                #[allow(dead_code)]
                pub fn is_empty(&self) -> bool {
                    self.0.is_empty()
                }

                pub fn into_raw(self) -> Vec<[u8; 32]> {
                    self.0
                }
            }
        )*
    };

    ($name: ident, $key: ident, $val: ident) => {
        #[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub std::collections::BTreeMap<$key, $val>);

        impl IntoIterator for $name {
            type Item = ($key, $val);
            type IntoIter = std::collections::btree_map::IntoIter<$key, $val>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        impl $name {
            pub fn insert(&mut self, key: $key, value: $val) {
                self.0.insert(key, value);
            }
        }
    }
}

pub fn trace_using<T, R, F>(listener: &mut T, f: F) -> R
where
    T: evm_tracing::EventListener
        + runtime_tracing::EventListener
        + gas_tracing::EventListener
        + 'static,
    F: FnOnce() -> R,
{
    let mut evm_listener = SharedMutableReference::new(listener);
    let mut runtime_listener = evm_listener.clone();
    let mut gas_listener = evm_listener.clone();

    gas_tracing::using(&mut gas_listener, || {
        runtime_tracing::using(&mut runtime_listener, || {
            evm_tracing::using(&mut evm_listener, f)
        })
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
            Event::Call { .. } | Event::Create { .. } | Event::PrecompileSubcall { .. } => {
                self.current.depth += 1;
            }
            Event::Exit {
                reason: _,
                return_value,
            } => {
                // If the depth is not zero then an error must have occurred to exit early.
                if !self.current.depth == 0 {
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
                context: _,
                opcode,
                position,
                stack,
                memory,
            } => {
                self.current.opcode = opcode;
                if let Ok(pc) = position {
                    self.current.program_counter = *pc as u32;
                }
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
                        // Step completed, push current log into the record
                        self.logs.push(self.current.clone());
                        // Current sub-call completed, reduce depth by 1
                        self.current.depth -= 1;

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
                        if opcode == &Opcode::SLOAD || opcode == &Opcode::SSTORE {
                            self.logs.push(self.current.clone());
                        }
                    }
                }
            }

            Event::SLoad {
                address: _,
                index,
                value,
            } => {
                self.current
                    .storage
                    .insert(LogStorageKey(index.0), LogStorageValue(value.0));
                self.logs.push(self.current.clone());
            }

            Event::SStore {
                address: _,
                index,
                value,
            } => {
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
            Event::RecordCost { cost, snapshot: _ } => {
                self.current.gas_cost = cost;
            }
            Event::RecordDynamicCost {
                gas_cost,
                memory_gas,
                gas_refund: _,
                snapshot: _,
            } => {
                // In SputnikVM memory gas is cumulative (ie this event always shows the total)
                // gas spent on memory up to this point. But geth traces simply
                // show how much gas each step took, regardless of how that gas
                // was used. So if this step caused an increase to the
                // memory gas then we need to record that.
                let memory_cost_diff = if memory_gas > self.current_memory_gas {
                    memory_gas - self.current_memory_gas
                } else {
                    0
                };
                self.current_memory_gas = memory_gas;
                self.current.gas_cost = gas_cost + memory_cost_diff;
            }
            Event::RecordRefund {
                refund: _,
                snapshot,
            } => {
                // This one seems to show up at the end of a transaction, so it
                // can be used to set the total gas used.
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
        TransactionTrace::new(self.gas_used, self.failed, self.output, Logs(self.logs))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceLog {
    pub depth:           u32,
    pub error:           Option<String>,
    pub gas:             u64,
    pub gas_cost:        u64,
    pub memory:          LogMemory,
    pub opcode:          Opcode,
    pub program_counter: u32,
    pub stack:           LogStack,
    pub storage:         LogStorage,
}

impl Default for TraceLog {
    fn default() -> Self {
        Self {
            depth:           Default::default(),
            error:           Default::default(),
            gas:             Default::default(),
            gas_cost:        Default::default(),
            memory:          Default::default(),
            opcode:          Opcode::STOP,
            program_counter: Default::default(),
            stack:           Default::default(),
            storage:         Default::default(),
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TransactionTrace {
    gas:          u64,
    failed:       bool,
    return_value: Vec<u8>,
    struct_logs:  Logs,
}

impl TransactionTrace {
    pub fn new(
        gas: u64,
        failed: bool,
        return_value: Vec<u8>,
        struct_logs: Logs,
    ) -> TransactionTrace {
        Self {
            gas,
            failed,
            return_value,
            struct_logs,
        }
    }

    pub fn gas(&self) -> u64 {
        self.gas
    }

    pub fn result(&self) -> &[u8] {
        self.return_value.as_slice()
    }

    pub fn logs(&self) -> &Logs {
        &self.struct_logs
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct StepTransactionTrace {
    inner: TransactionTrace,
    step:  usize,
}

#[allow(dead_code)]
impl StepTransactionTrace {
    pub fn new(transaction_trace: TransactionTrace) -> Self {
        Self {
            inner: transaction_trace,
            step:  0,
        }
    }

    pub fn step(&mut self) -> Option<&TraceLog> {
        if self.step > self.inner.struct_logs.len() {
            None
        } else {
            self.step += 1;
            Some(&self.inner.struct_logs[self.step])
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Logs(pub Vec<TraceLog>);

impl Logs {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Index<usize> for Logs {
    type Output = TraceLog;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for Logs {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = TraceLog;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

trace_type!(LogStorageKey, LogStorageValue,);
trace_type!(Vec, LogMemory, LogStack);
trace_type!(LogStorage, LogStorageKey, LogStorageValue);

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
