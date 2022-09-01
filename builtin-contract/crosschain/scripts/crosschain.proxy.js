const { ethers } = require("hardhat");
const util = require("ethereumjs-util");
const fs = require("fs");
const ABI = require('../artifacts/contracts/crosschain.sol/CrossChain.json');
const ERC1967Proxy = require('@openzeppelin/upgrades-core/artifacts/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol/ERC1967Proxy.json');

const private_key = Buffer.from("37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d", "hex");
const sender = util.privateToAddress(private_key);

const metadata = '0xb00d616b820c39619ee29e5144d0226cf8b5c15a'; // nonce = 2 (proxy)
const wckb = '0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352'; // nonce = 1

function address(value) {
    let hexed = value.toString("hex");
    if (hexed.length > 40) {
        hexed = hexed.substring(hexed.length - 40);
    }
    return "0x" + hexed;
}

async function main() {
    const export_json = {
        Implementation: {
            nonce: 3,
            data: [],
            address: '',
        },
        Proxy: {
            nonce: 5,
            data: [],
            address: '',
        },
    };

    // export MetadataManager implementation deployment data
    const impl_factory = await ethers.getContractFactory("CrossChain");
    const impl = impl_factory.getDeployTransaction();
    export_json.Implementation.data = Array.from(Buffer.from(impl.data.substring(2), 'hex'));

    // generate MetadataManager contract address and `construct` abi data
    const impl_stream = util.rlp.encode([sender, export_json.Implementation.nonce]);
    const impl_address = address(util.keccak256(impl_stream));
    const impl_abi = new ethers.utils.Interface(ABI.abi);
    const impl_construct_data = impl_abi.encodeFunctionData('construct', [metadata, wckb]);

    // export MetadataManager proxy deployment data
    const proxy_factory = await ethers.getContractFactory(ERC1967Proxy.abi, ERC1967Proxy.bytecode);
    const proxy = proxy_factory.getDeployTransaction(impl_address, impl_construct_data);
    export_json.Proxy.data = Array.from(Buffer.from(proxy.data.substring(2), 'hex'));

    // record two contracts addresses
    const proxy_stream = util.rlp.encode([sender, export_json.Proxy.nonce]);
    const proxy_address = address(util.keccak256(proxy_stream));
    export_json.Implementation.address = impl_address;
    export_json.Proxy.address = proxy_address;

    // export
    fs.writeFileSync(__dirname + "/crosschain.proxy.json", JSON.stringify(export_json, null, 2));
    console.log(`impl: ${impl_address}, proxy: ${proxy_address}`);
    console.log(`export deployment info to '${__dirname}/crosschain.proxy.json'`);
}

main().then(() => {
    process.exit(0)
});