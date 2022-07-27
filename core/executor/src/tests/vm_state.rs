use ethers_core::utils::hex;
use protocol::types::{Bytes, TransactionAction};
use protocol::types::{
    Eip1559Transaction, MemoryAccount, SignatureComponents, SignedTransaction, UnsignedTransaction,
    UnverifiedTransaction,
};
use protocol::types::{H160, H256, U256};
use serde::{Deserialize, Deserializer};
use std::{collections::BTreeMap, io::BufReader, mem::size_of, str::FromStr};

pub const BLOCK_INFO: &str = include_str!("../../res/vmTests/blockInfo.json");
pub const CALL_DATA_COPY: &str = include_str!("../../res/vmTests/calldatacopy.json");
pub const CALL_DATA_LOAD: &str = include_str!("../../res/vmTests/calldataload.json");
pub const CALL_DATA_SIZE: &str = include_str!("../../res/vmTests/calldatasize.json");
pub const DUP: &str = include_str!("../../res/vmTests/envInfo.json");
pub const ENV_INFO: &str = include_str!("../../res/vmTests/dup.json");
pub const PUSH: &str = include_str!("../../res/vmTests/envInfo.json");
pub const RANDOM: &str = include_str!("../../res/vmTests/push.json");
pub const SHA3: &str = include_str!("../../res/vmTests/random.json");
pub const SUICIDE: &str = include_str!("../../res/vmTests/sha3.json");
pub const SWAP: &str = include_str!("../../res/vmTests/suicide.json");

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_str(&s).map_err(serde::de::Error::custom)
}

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
                H256::from_str(&feel_zeros(v).unwrap()).expect("Can not parse account storage key"),
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
#[derive(Debug)]
pub enum NetworkType {
    Istanbul,
    Berlin,
    London,
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
            network => Err(format!("Not known network type, {}", network)),
        }
        .map_err(serde::de::Error::custom)
    }
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

impl TryInto<MemoryAccount> for AccountState {
    type Error = ();

    fn try_into(self) -> Result<MemoryAccount, Self::Error> {
        Ok(MemoryAccount {
            balance: self.balance,
            code:    self.code,
            nonce:   self.nonce,
            storage: self.storage,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallTransaction {
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub data:      Vec<u8>,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_limit: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_price: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub nonce:     U256,
    #[serde(deserialize_with = "deserialize_h160")]
    pub sender:    H160,
    #[serde(deserialize_with = "deserialize_h160")]
    pub to:        H160,
    #[serde(deserialize_with = "deserialize_u256")]
    pub value:     U256,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub r:         Vec<u8>,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub s:         Vec<u8>,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub v:         Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    #[serde(deserialize_with = "deserialize_h160")]
    pub coinbase:   H160,
    #[serde(deserialize_with = "deserialize_u256")]
    pub difficulty: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_limit:  U256,
    #[serde(deserialize_with = "deserialize_h256")]
    pub hash:       H256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub number:     U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub timestamp:  U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    block_header: BlockHeader,
    transactions: Vec<CallTransaction>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    #[serde(deserialize_with = "deserialize_accounts")]
    pre:                  BTreeMap<H160, AccountState>,
    network:              NetworkType,
    genesis_block_header: BlockHeader,
    blocks:               Vec<Block>,
    #[serde(deserialize_with = "deserialize_accounts")]
    post_state:           BTreeMap<H160, AccountState>,
}

pub trait TestEvmState: Sized {
    fn init_state() -> Self;

    fn try_apply_network_type(self, net_type: NetworkType) -> Result<Self, String>;

    fn try_apply_accounts<I>(self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (H160, AccountState)>;

    fn try_apply_block_header(self, block_header: BlockHeader) -> Result<Self, String>;

    fn try_apply_transaction(self, tx: CallTransaction) -> Result<Self, String>;

    fn validate_account(&self, address: H160, account: AccountState) -> Result<(), String>;

    fn try_apply_block(mut self, block: Block) -> Result<Self, String> {
        self = self.try_apply_block_header(block.block_header)?;
        for transaction in block.transactions {
            self = self.try_apply_transaction(transaction)?;
        }

        Ok(self)
    }

    fn try_apply_blocks<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = Block>,
    {
        for block in iter {
            self = self.try_apply_block(block)?;
        }
        Ok(self)
    }

    fn validate_accounts<I>(&self, iter: I) -> i32
    where
        I: Iterator<Item = (H160, AccountState)>,
    {
        let mut sum = 0;
        for (address, account) in iter {
            self.validate_account(address, account)
                .unwrap_or_else(|err| {
                    println!("{}", err);
                    sum += 1
                });
        }
        sum
    }
}

pub fn run_evm_test<State: TestEvmState>(test: &str) -> i32 {
    let mut sum = 0;
    let reader = BufReader::new(test.as_bytes());

    let test: BTreeMap<String, TestCase> =
        serde_json::from_reader(reader).expect("Parse test cases failed");

    for (test_name, test_case) in test {
        println!("\nRunning test: {} ...", test_name);

        let state = State::init_state()
            .try_apply_network_type(test_case.network)
            .unwrap()
            .try_apply_accounts(test_case.pre.into_iter())
            .unwrap()
            .try_apply_block_header(test_case.genesis_block_header)
            .unwrap()
            .try_apply_blocks(test_case.blocks.into_iter())
            .unwrap();
        let num = state.validate_accounts(test_case.post_state.into_iter());

        sum += num;
    }
    sum
}

pub fn run_evm_tests<State: TestEvmState>() {
    let tests = vec![
        BLOCK_INFO,
        CALL_DATA_COPY,
        CALL_DATA_LOAD,
        CALL_DATA_SIZE,
        DUP,
        ENV_INFO,
        PUSH,
        RANDOM,
        SHA3,
        SUICIDE,
        SWAP,
    ];
    let mut total = 0;
    for test in tests {
        let sum = run_evm_test::<State>(test);
        total += sum;
    }
    println!("**********************************************************");
    println!(
        "evm compatibility test result: total {} test cases failed.",
        total
    );
    println!("**********************************************************");
}

pub fn mock_signed_tx(tx: CallTransaction) -> SignedTransaction {
    let utx = UnverifiedTransaction {
        unsigned:  UnsignedTransaction::Eip1559(Eip1559Transaction {
            nonce:                    tx.nonce,
            gas_limit:                tx.gas_limit,
            max_priority_fee_per_gas: U256::one(),
            gas_price:                tx.gas_price,
            action:                   TransactionAction::Create,
            value:                    tx.value,
            data:                     Bytes::copy_from_slice(&tx.data),
            access_list:              vec![],
        }),
        chain_id:  5u64,
        hash:      H256::default(),
        signature: Some(SignatureComponents {
            standard_v: tx.v[0],
            r:          Bytes::copy_from_slice(&tx.r),
            s:          Bytes::copy_from_slice(&tx.s),
        }),
    }
    .calc_hash();
    SignedTransaction {
        transaction: utx,
        sender:      tx.sender,
        public:      None,
    }
}
