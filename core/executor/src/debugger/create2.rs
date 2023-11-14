use protocol::traits::Backend;
use protocol::types::{
    LegacyTransaction, SignedTransaction, TransactionAction, UnsignedTransaction,
    UnverifiedTransaction, H160, H256, MAX_BLOCK_GAS_LIMIT, U256,
};
use protocol::{codec::hex_decode, tokio};

use crate::debugger::EvmDebugger;

#[tokio::test(flavor = "multi_thread")]
async fn test_create2_gas() {
    let init_balance: U256 = 10000000000000000u64.into();
    let sender =
        H160::from_slice(&hex_decode("0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352").unwrap());
    let mut debugger = EvmDebugger::new(vec![sender], init_balance, "free-space/db3");

    let stx = mock_create2_tx(debugger.nonce(sender), sender);
    let resp = debugger.exec(1, vec![stx]);

    // let resp = debugger.call(
    //     1,
    //     Some(sender),
    //     Some(resp.tx_resp[0].code_address.unwrap().into()),
    //     U256::zero(),
    //     hex_decode("107aa604").unwrap(),
    // );
    // println!("{:?}", resp);

    let gas_used = resp.tx_resp[0].gas_used;
    let after_balance = debugger.backend(1).basic(sender).balance;

    assert_eq!(gas_used * 8, (init_balance - after_balance).low_u64());
}

fn mock_create2_tx(nonce: U256, sender: H160) -> SignedTransaction {
    let tx = LegacyTransaction {
		nonce,
		gas_price: 8u64.into(),
		gas_limit: MAX_BLOCK_GAS_LIMIT.into(),
		action: 				  TransactionAction::Create,
		value: U256::zero(),
		data: hex_decode("60806040523480156200001157600080fd5b506040516200002090620000ef565b604051809103906000f0801580156200003d573d6000803e3d6000fd5b506000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506040516200008b90620000fd565b604051809103906000f080158015620000a8573d6000803e3d6000fd5b50600160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506200010b565b61064d806200198583390190565b610be28062001fd283390190565b61186a806200011b6000396000f3fe608060405234801561001057600080fd5b50600436106100625760003560e01c8063107aa604146100675780631cf41a81146100715780634d290d3e1461008d578063d62d311514610097578063efbcc6b4146100b5578063f95f2142146100bf575b600080fd5b61006f6100ef565b005b61008b6004803603810190610086919061111e565b6103f2565b005b61009561059d565b005b61009f610821565b6040516100ac9190611354565b60405180910390f35b6100bd610b79565b005b6100d960048036038101906100d4919061111e565b610e84565b6040516100e691906112e7565b60405180910390f35b6000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381871cbc3360016040518363ffffffff1660e01b815260040161014f929190611302565b60006040518083038186803b15801561016757600080fd5b505afa15801561017b573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f820116820180604052508101906101a491906110d5565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d083650bef7717b3b06040518363ffffffff1660e01b815260040161020b92919061139f565b602060405180830381600087803b15801561022557600080fd5b505af1158015610239573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061025d91906110a8565b905060008190508073ffffffffffffffffffffffffffffffffffffffff16639cb8a26a6040518163ffffffff1660e01b8152600401600060405180830381600087803b1580156102ac57600080fd5b505af11580156102c0573d6000803e3d6000fd5b505050506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d085650bef7717b3b06040518363ffffffff1660e01b815260040161032992919061139f565b602060405180830381600087803b15801561034357600080fd5b505af1158015610357573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061037b91906110a8565b9050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16146103ec576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016103e3906114ea565b60405180910390fd5b50505050565b6000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381871cbc33846040518363ffffffff1660e01b815260040161045192919061132b565b60006040518083038186803b15801561046957600080fd5b505afa15801561047d573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f820116820180604052508101906104a691906110d5565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d08362bbe2236040518363ffffffff1660e01b815260040161050a92919061136f565b602060405180830381600087803b15801561052457600080fd5b505af1158015610538573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061055c91906110a8565b90507f35e3089d2d7a4ec5640ff07c04690a010b43060749c201136090c4ef49967c1d60018260405161059092919061142f565b60405180910390a1505050565b6000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381871cbc3360016040518363ffffffff1660e01b81526004016105fd929190611302565b60006040518083038186803b15801561061557600080fd5b505afa158015610629573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f8201168201806040525081019061065291906110d5565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d0836501318be8c52b6040518363ffffffff1660e01b81526004016106b99291906113cf565b602060405180830381600087803b1580156106d357600080fd5b505af11580156106e7573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061070b91906110a8565b905060008190508073ffffffffffffffffffffffffffffffffffffffff16639cb8a26a6040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561075a57600080fd5b505af115801561076e573d6000803e3d6000fd5b505050507f35e3089d2d7a4ec5640ff07c04690a010b43060749c201136090c4ef49967c1d6003836040516107a4929190611481565b60405180910390a1600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16141561081c576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004016108139061150a565b60405180910390fd5b505050565b600080600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381871cbc3360016040518363ffffffff1660e01b8152600401610882929190611302565b60006040518083038186803b15801561089a57600080fd5b505afa1580156108ae573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f820116820180604052508101906108d791906110d5565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166394ca2cb58360016040518363ffffffff1660e01b81526004016109399291906113ff565b60206040518083038186803b15801561095157600080fd5b505afa158015610965573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061098991906110a8565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d08460016040518363ffffffff1660e01b81526004016109eb9291906113ff565b602060405180830381600087803b158015610a0557600080fd5b505af1158015610a19573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a3d91906110a8565b90508073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614610aad576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610aa49061152a565b60405180910390fd5b600081905060018173ffffffffffffffffffffffffffffffffffffffff1663243dc8da6040518163ffffffff1660e01b815260040160206040518083038186803b158015610afa57600080fd5b505afa158015610b0e573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610b32919061114b565b14610b72576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610b69906114aa565b60405180910390fd5b5050505090565b6000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381871cbc3360016040518363ffffffff1660e01b8152600401610bd9929190611302565b60006040518083038186803b158015610bf157600080fd5b505afa158015610c05573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f82011682018060405250810190610c2e91906110d5565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d08362bbe2236040518363ffffffff1660e01b8152600401610c9292919061136f565b602060405180830381600087803b158015610cac57600080fd5b505af1158015610cc0573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610ce491906110a8565b90507f35e3089d2d7a4ec5640ff07c04690a010b43060749c201136090c4ef49967c1d600182604051610d1892919061142f565b60405180910390a16000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16639c4ae2d08462bbe2236040518363ffffffff1660e01b8152600401610d8292919061136f565b602060405180830381600087803b158015610d9c57600080fd5b505af1158015610db0573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610dd491906110a8565b90507f35e3089d2d7a4ec5640ff07c04690a010b43060749c201136090c4ef49967c1d600282604051610e08929190611458565b60405180910390a1600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614610e7f576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401610e76906114ca565b60405180910390fd5b505050565b600080600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381871cbc33856040518363ffffffff1660e01b8152600401610ee492919061132b565b60006040518083038186803b158015610efc57600080fd5b505afa158015610f10573d6000803e3d6000fd5b505050506040513d6000823e3d601f19601f82011682018060405250810190610f3991906110d5565b90506000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166394ca2cb58362bbe2236040518363ffffffff1660e01b8152600401610f9d92919061136f565b60206040518083038186803b158015610fb557600080fd5b505afa158015610fc9573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610fed91906110a8565b90508092505050919050565b600061100c6110078461156f565b61154a565b90508281526020810184848401111561102857611027611719565b5b611033848285611681565b509392505050565b60008151905061104a81611806565b92915050565b600082601f83011261106557611064611714565b5b8151611075848260208601610ff9565b91505092915050565b60008135905061108d8161181d565b92915050565b6000815190506110a28161181d565b92915050565b6000602082840312156110be576110bd611723565b5b60006110cc8482850161103b565b91505092915050565b6000602082840312156110eb576110ea611723565b5b600082015167ffffffffffffffff8111156111095761110861171e565b5b61111584828501611050565b91505092915050565b60006020828403121561113457611133611723565b5b60006111428482850161107e565b91505092915050565b60006020828403121561116157611160611723565b5b600061116f84828501611093565b91505092915050565b611181816115cd565b82525050565b611190816115df565b82525050565b60006111a1826115a0565b6111ab81856115ab565b93506111bb818560208601611681565b6111c481611728565b840191505092915050565b6111d881611615565b82525050565b6111e781611627565b82525050565b6111f681611639565b82525050565b6112058161164b565b82525050565b6112148161165d565b82525050565b6112238161166f565b82525050565b6000611236600b836115bc565b915061124182611739565b602082019050919050565b6000611259600b836115bc565b915061126482611762565b60208201905091").unwrap().into(),
	};

    let utx = UnverifiedTransaction {
        unsigned:  UnsignedTransaction::Legacy(tx),
        signature: None,
        chain_id:  Some(5u64),
        hash:      H256::default(),
    };

    SignedTransaction {
        sender,
        transaction: utx,
        public: Some(Default::default()),
    }
}
