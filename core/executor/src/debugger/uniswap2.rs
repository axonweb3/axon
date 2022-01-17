use super::*;
use protocol::codec::hex_decode;
use protocol::tokio;
use protocol::types::{Bytes, TransactionAction, H160};
use std::str::FromStr;

use ethabi_contract::use_contract;
use evm::{ExitReason, ExitSucceed};

use_contract!(factory, "./res/factory.abi");
use_contract!(router, "./res/router.abi");
use_contract!(weth, "./res/weth.abi");
use_contract!(erc20, "./res/erc20.abi");

use erc20::constructor as erc20_constructor;
use erc20::functions as erc20_functions;
use factory::constructor as factory_constructor;
use router::constructor as router_constructor;
use router::functions as router_functions;
use std::fs::File;
use std::io::{BufReader, Read};
use weth::functions as weth_functions;

const PAIR_INIT_CODE_HASH: &str =
    "f8b6d1d8e3b05a01c9235ebfcb119fab138bc18a3baaf65e5405861e9a2a60c8";

fn read_code(path: &str) -> String {
    let file = File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    contents.trim().to_string()
}

fn construct_tx(action: TransactionAction, value: U256, data: Vec<u8>) -> Transaction {
    Transaction {
        nonce: U256::default(),
        max_priority_fee_per_gas: U256::default(),
        gas_price: U256::default(),
        gas_limit: 10000000000u64.into(),
        action,
        value,
        data: Bytes::from(data),
        access_list: Vec::new(),
    }
}

// factory contract https://cn.etherscan.com/address/0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f#code
fn deploy_factory(
    debugger: &mut EvmDebugger,
    sender: H160,
    block_number: &mut u64,
    setter: H160,
) -> H160 {
    println!("######## Deploy factory contract");
    let factory_code = hex_decode(&read_code("./res/factory_code.txt")).unwrap();
    let deploy_data = factory_constructor(factory_code, setter);

    let tx = construct_tx(TransactionAction::Create, U256::default(), deploy_data);
    let stx = mock_signed_tx(tx, sender);

    let resp = debugger.exec(*block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );

    *block_number += 1;

    let code_address = resp.tx_resp[0].code_address.unwrap().0;
    H160::from_slice(&code_address[12..])
}

// weth contract https://cn.etherscan.com/address/0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2#code
fn deploy_weth(debugger: &mut EvmDebugger, sender: H160, block_number: &mut u64) -> H160 {
    println!("######## Deploy WETH contract");
    let weth_code = hex_decode(&read_code("./res/weth_code.txt")).unwrap();

    let tx = construct_tx(TransactionAction::Create, U256::default(), weth_code);
    let stx = mock_signed_tx(tx, sender);

    let resp = debugger.exec(*block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );

    *block_number += 1;

    let code_address = resp.tx_resp[0].code_address.unwrap().0;
    H160::from_slice(&code_address[12..])
}

// router contract https://cn.etherscan.com/address/0x7a250d5630b4cf539739df2c5dacb4c659f2488d#code
fn deploy_router(
    debugger: &mut EvmDebugger,
    sender: H160,
    block_number: &mut u64,
    factory_address: H160,
    weth_address: H160,
) -> H160 {
    println!("######## Deploy router contract");
    let router_code = hex_decode(&read_code("./res/router_code.txt")).unwrap();
    let deploy_data = router_constructor(router_code, factory_address, weth_address);

    let tx = construct_tx(TransactionAction::Create, U256::default(), deploy_data);
    let stx = mock_signed_tx(tx, sender);

    let resp = debugger.exec(*block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );

    *block_number += 1;

    let code_address = resp.tx_resp[0].code_address.unwrap().0;
    H160::from_slice(&code_address[12..])
}

// erc20 contract https://cn.etherscan.com/address/0x38A6741251e849ddaD46619b720a6970f73aD25A#code
fn deploy_erc20(
    debugger: &mut EvmDebugger,
    sender: H160,
    block_number: &mut u64,
    name: String,
    symbol: String,
    owner: H160,
) -> H160 {
    println!("######## Deploy erc20 contract");
    let erc20_code = hex_decode(&read_code("./res/erc20_code.txt")).unwrap();
    let deploy_data = erc20_constructor(erc20_code, name, symbol, owner);

    let tx = construct_tx(TransactionAction::Create, U256::default(), deploy_data);
    let stx = mock_signed_tx(tx, sender);

    let resp = debugger.exec(*block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );

    *block_number += 1;

    let code_address = resp.tx_resp[0].code_address.unwrap().0;
    H160::from_slice(&code_address[12..])
}

fn get_pair_address(token_a: H160, token_b: H160, factory: H160) -> H160 {
    let (token_a, token_b) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    let mut data = token_a.as_bytes().to_vec();
    data.extend_from_slice(token_b.as_bytes());
    let tmp = Hasher::digest(data);

    let mut data = hex_decode("ff").unwrap();
    let pair_init_code_hash = hex_decode(PAIR_INIT_CODE_HASH).unwrap();
    data.extend_from_slice(factory.as_bytes());
    data.extend_from_slice(tmp.as_bytes());
    data.extend_from_slice(&pair_init_code_hash);

    let hash = Hasher::digest(data);
    H160::from_slice(&hash.0[12..])
}

#[tokio::test(flavor = "multi_thread")]
async fn test_uniswap2_add_liquidity() {
    let distribution_address =
        H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5fb5").unwrap();
    let distribution_amount: U256 = 1234560000000000000000000u128.into();

    let mut debugger = EvmDebugger::new(distribution_address, distribution_amount);
    let sender = distribution_address;
    let mut block_number = 1;

    let factory_address = deploy_factory(&mut debugger, sender, &mut block_number, sender); // factory's contract address is 0x6beb69f8bb038f1e9e4291ca3e5a0cddf633e796
    let weth_address = deploy_weth(&mut debugger, sender, &mut block_number); // weth's contract address is 0x2d7b72c32ff49908b3dd12434d6c78338385f2ef
    let router_address = deploy_router(
        &mut debugger,
        sender,
        &mut block_number,
        factory_address,
        weth_address,
    );

    // deploy 1st erc20
    let erc20_name_0 = "TT".to_string();
    let erc20_symbol_0 = "18".to_string();
    let erc20_address_0 = deploy_erc20(
        &mut debugger,
        sender,
        &mut block_number,
        erc20_name_0,
        erc20_symbol_0,
        sender,
    );

    // deploy 2rd erc20
    let erc20_name_1 = "WW".to_string();
    let erc20_symbol_1 = "18".to_string();
    let erc20_address_1 = deploy_erc20(
        &mut debugger,
        sender,
        &mut block_number,
        erc20_name_1,
        erc20_symbol_1,
        sender,
    );

    // Approve router max allowance of 1st erc20
    println!(
        "######## Approve router max allowance of 1st erc20, which is essential for addLiquidity"
    );
    let call_approve_code = erc20_functions::approve::encode_input(router_address, U256::MAX);
    let tx = construct_tx(
        TransactionAction::Call(erc20_address_0),
        U256::default(),
        call_approve_code.clone(),
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
    block_number += 1;

    // Approve router max allowance of 2rd erc20
    println!(
        "######## Approve router max allowance of 2rd erc20, which is essential for addLiquidity"
    );
    let tx = construct_tx(
        TransactionAction::Call(erc20_address_1),
        U256::default(),
        call_approve_code,
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
    block_number += 1;

    println!("######## Call addLiquidity");
    let token_a = erc20_address_0;
    let token_b = erc20_address_1;
    let amount_a_designed: U256 = 50000_000000000000000000u128.into();
    let amount_b_designed: U256 = 10000_000000000000000000u128.into();
    let amount_a_min: U256 = 49000_000000000000000000u128.into();
    let amount_b_min: U256 = 9000_000000000000000000u128.into();
    let to = sender;
    let deadline: U256 = (time_now() + 1_000_000).into();
    let call_add_liquidity_code = router_functions::add_liquidity::encode_input(
        token_a,
        token_b,
        amount_a_designed,
        amount_b_designed,
        amount_a_min,
        amount_b_min,
        to,
        deadline,
    );
    let tx = construct_tx(
        TransactionAction::Call(router_address),
        U256::default(),
        call_add_liquidity_code,
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_uniswap2_add_liquidity_eth() {
    let distribution_address =
        H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5fb5").unwrap();
    let distribution_amount: U256 = 1234560000000000000000000u128.into();

    let mut debugger = EvmDebugger::new(distribution_address, distribution_amount);
    let sender = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5fb5").unwrap();
    let mut block_number = 1;

    let factory_address = deploy_factory(&mut debugger, sender, &mut block_number, sender); // factory's contract address is 0x6beb69f8bb038f1e9e4291ca3e5a0cddf633e796
    let weth_address = deploy_weth(&mut debugger, sender, &mut block_number); // weth's contract address is 0x2d7b72c32ff49908b3dd12434d6c78338385f2ef
    let router_address = deploy_router(
        &mut debugger,
        sender,
        &mut block_number,
        factory_address,
        weth_address,
    );

    println!("######## Call router's factory(), which is essential for addLiquidityETH, must be called before deploying erc20 contract");
    let call_factory_code = router_functions::factory::encode_input();
    let tx = construct_tx(
        TransactionAction::Call(router_address),
        U256::default(),
        call_factory_code,
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
    block_number += 1;

    // deploy erc20
    let erc20_name = "TT".to_string();
    let erc20_symbol = "18".to_string();
    let erc20_address = deploy_erc20(
        &mut debugger,
        sender,
        &mut block_number,
        erc20_name,
        erc20_symbol,
        sender,
    );

    // Approve router max allowance of 1st erc20
    println!("######## Approve router max allowance of erc20, which is essential for addLiquidity");
    let call_approve_code = erc20_functions::approve::encode_input(router_address, U256::MAX);
    let tx = construct_tx(
        TransactionAction::Call(erc20_address),
        U256::default(),
        call_approve_code,
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
    block_number += 1;

    // deposit WETH
    println!("######## Deposit WETH, which is essential for addLiquidityETH");
    let value = distribution_amount;
    let tx = construct_tx(TransactionAction::Call(weth_address), value, vec![]);
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Stopped)
    );
    block_number += 1;

    println!("######## transfer WETH to pair_address, which is essential for addLiquidityETH");
    let pair_address = get_pair_address(erc20_address, weth_address, factory_address);
    println!("pair_address: {:?}", pair_address);
    let value = distribution_amount / 2;
    let call_transfer_code = weth_functions::transfer::encode_input(pair_address, value);
    let tx = construct_tx(
        TransactionAction::Call(weth_address),
        U256::default(),
        call_transfer_code,
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
    block_number += 1;

    println!("######## Call addLiquidityETH");
    let token = erc20_address;
    let amount_token_desired: U256 = 50000_000000000000000000u128.into();
    let amount_token_min: U256 = 49000_000000000000000000u128.into();
    let amount_eth_min: U256 = 10000_000000000000000000u128.into();
    let to = sender;
    let deadline: U256 = (time_now() + 1_000_000).into();
    let call_add_liquidity_code = router_functions::add_liquidity_eth::encode_input(
        token,
        amount_token_desired,
        amount_token_min,
        amount_eth_min,
        to,
        deadline,
    );
    let tx = construct_tx(
        TransactionAction::Call(router_address),
        U256::default(),
        call_add_liquidity_code,
    );
    let stx = mock_signed_tx(tx, sender);
    let resp = debugger.exec(block_number, vec![stx]);
    println!("{:?}", resp);
    assert_eq!(
        resp.tx_resp[0].exit_reason,
        ExitReason::Succeed(ExitSucceed::Returned)
    );
}
