use crate::AxonExecutor;
use ethers_core::utils::hex;
use evm::{ExitReason, ExitSucceed};
use protocol::types::{MemoryAccount, MemoryVicinity};
use protocol::{
    traits::Executor,
    types::{MemoryBackend, H160, H256, U256},
};
use serde::{Deserialize, Deserializer};
use std::{collections::BTreeMap, io::BufReader, mem::size_of, str::FromStr};

use super::gen_vicinity;

fn deserialize_u64_vec<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<String> = Vec::deserialize(deserializer)?;
    let res: Vec<u64> = vec
        .into_iter()
        .map(|s| {
            u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("unable to parse to u64")
        })
        .collect();
    Ok(res)
}

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_str(&s).map_err(serde::de::Error::custom)
}

// fn deserialize_hex_u256_vec<'de, D>(deserializer: D) -> Result<Vec<U256>,
// D::Error> where
//     D: Deserializer<'de>,
// {
//     let vec: Vec<String> = Vec::deserialize(deserializer)?;
//     let res: Vec<U256> = vec
//         .into_iter()
//         .map(|s| U256::from_str(&s).map_err(serde::de::Error::custom))
//         .collect();
//     Ok(res)
// }

fn deserialize_h256<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    H256::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_h160<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    H160::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_hex_data<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    hex::decode(
        s[0..2]
            .eq("0x")
            .then(|| s[2..].to_string())
            .ok_or("Missing '0x' prefix for hex data")
            .map_err(serde::de::Error::custom)?,
    )
    .map_err(serde::de::Error::custom)
}

fn deserialize_hex_data_vec<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<String> = Vec::deserialize(deserializer)?;
    let res = vec
        .into_iter()
        .map(|v| {
            hex::decode(
                v[0..2]
                    .eq("0x")
                    .then(|| v[2..].to_string())
                    .ok_or("Missing '0x' prefix for hex data")
                    .map_err(|err| err.to_string())
                    .unwrap(),
            )
            .map_err(|err| err.to_string())
            .unwrap()
        })
        .collect::<Vec<Vec<u8>>>();
    Ok(res)
}

fn deserialize_account_storage<'de, D>(deserializer: D) -> Result<BTreeMap<H256, H256>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = <BTreeMap<String, String>>::deserialize(deserializer)?;
    let feel_zeros = |mut val: String| -> Result<String, String> {
        val = val[0..2]
            .eq("0x")
            .then(|| val[2..].to_string())
            .ok_or("Missing '0x' prefix for hex data")?;

        while val.len() < size_of::<H256>() * 2 {
            val = "00".to_string() + &val;
        }
        val = "0x".to_string() + &val;
        Ok(val)
    };
    Ok(map
        .into_iter()
        .map(|(k, v)| {
            (
                H256::from_str(&feel_zeros(k).unwrap()).expect("Can not parse account storage key"),
                H256::from_str(&feel_zeros(v).unwrap())
                    .expect("Can not parse account storage value"),
            )
        })
        .collect())
}

fn deserialize_accounts<'de, D>(deserializer: D) -> Result<BTreeMap<H160, AccountState>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = <BTreeMap<String, AccountState>>::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (H160::from_str(&k).unwrap(), v))
        .collect())
}

pub enum NetworkType {
    Istanbul,
    Berlin,
    London,
    // Merge,
}

impl<'de> Deserialize<'de> for NetworkType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Istanbul" => Ok(Self::Istanbul),
            "Berlin" => Ok(Self::Berlin),
            "London" => Ok(Self::London),
            // "Merge" => Ok(Self::Merge),
            network => Err(format!("Not known network type, {}", network)),
        }
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Debug)]
pub struct Indexes {
    pub data:  u128,
    pub gas:   u128,
    pub value: u128,
}

#[derive(Deserialize, Debug)]
pub struct PostTx {
    #[serde(deserialize_with = "deserialize_h256")]
    pub hash:    H256,
    pub indexes: Indexes,
    #[serde(deserialize_with = "deserialize_h256")]
    pub logs:    H256,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub txbytes: Vec<u8>, // todo? sure?
}

#[derive(Deserialize, Debug)]
pub struct AccountState {
    #[serde(deserialize_with = "deserialize_u256")]
    pub balance: U256,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub code:    Vec<u8>,
    #[serde(deserialize_with = "deserialize_u256")]
    pub nonce:   U256,
    #[serde(deserialize_with = "deserialize_account_storage")]
    pub storage: BTreeMap<H256, H256>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Env {
    #[serde(deserialize_with = "deserialize_u256")]
    pub current_base_fee:   U256,
    #[serde(deserialize_with = "deserialize_h160")]
    pub current_coinbase:   H160,
    #[serde(deserialize_with = "deserialize_u256")]
    pub current_difficulty: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub current_gas_limit:  U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub current_number:     U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub current_timestamp:  U256,
    #[serde(deserialize_with = "deserialize_h256")]
    pub previous_hash:      H256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallTransaction {
    #[serde(deserialize_with = "deserialize_hex_data_vec")]
    pub data:       Vec<Vec<u8>>,
    #[serde(deserialize_with = "deserialize_u64_vec")]
    pub gas_limit:  Vec<u64>,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_price:  U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub nonce:      U256,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub secret_key: Vec<u8>, // todo or h256?
    #[serde(deserialize_with = "deserialize_h160")]
    pub sender:     H160,
    #[serde(deserialize_with = "deserialize_h160")]
    pub to:         H160,
    #[serde(deserialize_with = "deserialize_hex_data_vec")]
    pub value:      Vec<Vec<u8>>,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub data:       Vec<u8>,
    // pub gas_limit:  Vec<u8>,
    pub gas_limit:  u64,
    pub gas_price:  U256,
    pub nonce:      U256,
    pub secret_key: Vec<u8>, // todo?
    pub sender:     H160,
    pub to:         H160,
    pub value:      Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    env:         Env,
    #[serde(deserialize_with = "deserialize_accounts")]
    pre:         BTreeMap<H160, AccountState>,
    post:        BTreeMap<String, Vec<PostTx>>,
    transaction: CallTransaction,
}

pub trait EvmUnitTest: Sized {
    fn init_state() -> Self;

    fn try_apply_chain_id(self, id: U256) -> Result<Self, String>;

    fn try_apply_network_type(self, net_type: NetworkType) -> Result<Self, String>;

    fn try_apply_environment(self, env: Env) -> Result<Self, String>;

    fn try_apply_account(self, address: H160, account: AccountState) -> Result<Self, String>;

    fn try_apply_transaction(self, tx: Transaction) -> Result<Self, String>;

    // pre
    fn try_apply_accounts<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (H160, AccountState)>,
    {
        for (address, account) in iter {
            self = self.try_apply_account(address, account)?;
        }
        Ok(self)
    }

    fn try_apply_transactions(mut self, txs: CallTransaction) -> Result<Self, String> {
        for data in txs.data {
            let transaction = Transaction {
                data,
                gas_limit: txs.gas_limit[0],
                gas_price: txs.gas_price,
                nonce: txs.nonce,
                secret_key: txs.secret_key.clone(),
                sender: txs.sender,
                to: txs.to,
                value: txs.value[0].clone(),
            };
            self = self.try_apply_transaction(transaction)?;
        }

        Ok(self)
    }

    fn validate_post(&self, hash: H256, account: PostTx) -> Result<(), String>;

    fn validate_posts<I>(&self, iter: I) -> Result<(), String>
    where
        I: Iterator<Item = (String, Vec<PostTx>)>,
    {
        for (worktype, txs) in iter {
            if worktype != "London" {
                continue;
            }
            for tx in txs {
                self.validate_post(tx.hash, tx)?;
            }
        }
        Ok(())
    }
}

pub fn run_evm_test<State: EvmUnitTest>(test: &str) {
    let reader = BufReader::new(test.as_bytes());
    let test: BTreeMap<String, TestCase> =
        serde_json::from_reader(reader).expect("Parse test cases failed");

    for (test_name, test_case) in test {
        println!("\nRunning test: {} ...", test_name);
        let state = State::init_state()
            .try_apply_network_type(NetworkType::London)
            .unwrap()
            .try_apply_environment(test_case.env)
            .unwrap()
            .try_apply_accounts(test_case.pre.into_iter())
            .unwrap()
            .try_apply_transactions(test_case.transaction)
            .unwrap();

        state.validate_posts(test_case.post.into_iter()).unwrap();
    }
}

#[allow(unused_doc_comments)]
pub fn run_evm_tests<State: EvmUnitTest>() {
    use crate::tests::vm_path;
    let tests = vec![
        vm_path::BLOCK_INFO,
        vm_path::CALL_DATA_COPY,
        vm_path::CALL_DATA_LOAD,
        vm_path::CALL_DATA_SIZE,
        vm_path::DUP,
        vm_path::ENV_INFO,
        vm_path::PUSH,
        vm_path::RANDOM,
        vm_path::SHA3,
        vm_path::SUICIDE,
        vm_path::SWAP,
    ];

    for test in tests {
        run_evm_test::<State>(test);
    }
}

struct EvmUnitTestDebugger {
    vicinity: MemoryVicinity,
    state:    BTreeMap<H160, MemoryAccount>,
    executor: AxonExecutor,
}

impl EvmUnitTest for EvmUnitTestDebugger {
    fn init_state() -> Self {
        EvmUnitTestDebugger {
            vicinity: gen_vicinity(),
            state:    BTreeMap::new(),
            executor: AxonExecutor::default(),
        }
    }

    fn try_apply_chain_id(mut self, id: U256) -> Result<Self, String> {
        self.vicinity.chain_id = id;
        Ok(self)
    }

    fn try_apply_network_type(self, _: NetworkType) -> Result<Self, String> {
        Ok(self)
    }

    fn try_apply_environment(mut self, env: Env) -> Result<Self, String> {
        self.vicinity.block_number = env.current_number;
        self.vicinity.block_coinbase = env.current_coinbase;
        self.vicinity.block_timestamp = env.current_timestamp;
        self.vicinity.block_difficulty = env.current_difficulty;
        self.vicinity.block_gas_limit = env.current_gas_limit;
        Ok(self)
    }

    fn try_apply_account(mut self, address: H160, account: AccountState) -> Result<Self, String> {
        self.state.insert(address, MemoryAccount {
            nonce:   account.nonce,
            balance: account.balance,
            storage: account.storage,
            code:    account.code,
        });
        Ok(self)
    }

    fn try_apply_transaction(self, tx: Transaction) -> Result<Self, String> {
        let mut backend = MemoryBackend::new(&self.vicinity, self.state.clone());

        let r = self.executor.call(
            &mut backend,
            tx.gas_limit,
            Some(tx.sender),
            Some(tx.to),
            U256::from_big_endian(&tx.value),
            tx.data,
        );
        println!("{:?}", r.exit_reason);
        // assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
        assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Stopped));

        Ok(self)
    }

    fn validate_post(&self, _: H256, _: PostTx) -> Result<(), String> {
        Ok(())
    }

    fn validate_posts<I>(&self, iter: I) -> Result<(), String>
    where
        I: Iterator<Item = (String, Vec<PostTx>)>,
    {
        for (_, txs) in iter {
            for tx in txs {
                self.validate_post(tx.hash, tx)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn run_tests() {
        run_evm_tests::<EvmUnitTestDebugger>();
    }
}
