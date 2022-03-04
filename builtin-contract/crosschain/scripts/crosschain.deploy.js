const { ethers } = require("hardhat")
const { FeeMarketEIP1559Transaction } = require("@ethereumjs/tx")
const util = require("ethereumjs-util")
const fs = require("fs")
const private_key = Buffer.from("37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d", "hex")

async function export_deploy() {
    const crosschain = await ethers.getContractFactory("CrossChain")
    const tx = {
        "value": "0x0",
        "maxPriorityFeePerGas": "0x7d0",
        "maxFeePerGas": "0x7d0",
        "gasLimit": "0x3231303030",
        "nonce": "0x5",
        "data": crosschain.bytecode,
        "accessList": [],
        "chainId": 5,
        "type": 2
    }
    return FeeMarketEIP1559Transaction.fromTxData(tx).sign(private_key)
}

// caution: this method only generates mock transaction with mismatched signature to deploy to Axon genesis block
export_deploy().then(signed_tx => {
    const hex = (value, length) => {
        let hexed = value.toString("hex")
        while (length != null && hexed.length < length) {
            hexed = "0" + hexed
        }
        return "0x" + hexed
    }
    const deploy = {
        "transaction": {
            "unsigned": {
                "nonce": hex(signed_tx.nonce),
                "max_priority_fee_per_gas": hex(signed_tx.maxPriorityFeePerGas),
                "gas_price": hex(signed_tx.maxFeePerGas),
                "gas_limit": hex(signed_tx.gasLimit),
                "action": "Create",
                "value": hex(signed_tx.value),
                "data": Array.from(signed_tx.data),
                "access_list": signed_tx.accessList
            },
            "signature": {
                "r": hex(signed_tx.r, 64),
                "s": hex(signed_tx.s, 64),
                "standard_v": signed_tx.v.toNumber(),
            },
            "chain_id": signed_tx.chainId.toNumber(),
            "hash": hex(signed_tx.hash())
        },
        "sender": hex(util.privateToAddress(private_key)),
        "public": hex(util.privateToPublic(private_key)) 
    }
    const stream = util.rlp.encode([util.privateToAddress(private_key), signed_tx.nonce])
    const code_address = hex(util.keccak256(stream))
    fs.writeFileSync(__dirname + "/crosschain.deploy.json", JSON.stringify({ deploy, code_address }, null, 2))
    console.log(`export deployment info to '${__dirname}/crosschain.deploy.json'`)
    process.exit(0)
})
