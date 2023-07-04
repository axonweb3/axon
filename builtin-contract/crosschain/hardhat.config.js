const { getContractAddress } = require("ethers/lib/utils");
const { task } = require("hardhat/config");

require("@nomiclabs/hardhat-waffle");
require('@openzeppelin/hardhat-upgrades');
require('dotenv').config();

const wckb = '0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352';
// const crosschain = '0xf67bc4e50d1df92b0e4c61794a4517af6a995cb2';
const crosschain = '0xc4a3c40808d63e1ee18f7739f239dfa5bc92bfcd';


// This is a sample Hardhat task. To learn how to create your own go to
// https://hardhat.org/guides/create-task.html
task("accounts", "Prints the list of accounts", async (taskArgs, hre) => {
  const accounts = await hre.ethers.getSigners();

  for (const account of accounts) {
    console.log(account.address);
  }
});

task('deployTestToken', 'deploy test token on axon').addParam('private').setAction(async (taskArgs, hre) => {
  const signer = new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  const TestToken = await hre.ethers.getContractFactory('TestToken');
  let unsignedTx = TestToken.getDeployTransaction();
  unsignedTx = await signer.populateTransaction(unsignedTx);
  // unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
  console.log(getContractAddress(tx));
});

task('deployMirrorToken', 'deploy mirror token on axon').addParam('private').addParam('name').addParam('symbol').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer = new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  const MirrorToken = await hre.ethers.getContractFactory('MirrorToken');
  let unsignedTx = MirrorToken.getDeployTransaction(taskArgs.name, taskArgs.symbol, 18);
  unsignedTx = await signer.populateTransaction(unsignedTx);
  // unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
  console.log(getContractAddress(tx));
});

task('crossAt', 'cross at').addParam('to').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('lockAT', [taskArgs.to]),
    to: hre.ethers.utils.getAddress(crosschain),
    value: hre.ethers.utils.parseEther('50'),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  // unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('crossToken', 'cross token').addParam('to').addParam('token').addParam('amount').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('crossTokenToCKB', [taskArgs.to, taskArgs.token, hre.ethers.BigNumber.from(taskArgs.amount)]),//hre.ethers.utils.parseUnits(taskArgs.amount, 18)]),
    to: hre.ethers.utils.getAddress(crosschain),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  // unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('crossFromCKB', 'cross from ckb').addParam('to').addParam('token').addParam('ckb').addParam('sudt').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  const sUDTAmount = hre.ethers.BigNumber.from(taskArgs.sudt);
  const CKBAmount = hre.ethers.BigNumber.from(taskArgs.ckb);
  const txHash = hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes('xxx'));
  let unsignedTx = {
    data: abi.encodeFunctionData('crossFromCKB', [[[taskArgs.to, taskArgs.token, sUDTAmount, CKBAmount, txHash]], 0]),
    to: hre.ethers.utils.getAddress(crosschain),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  // unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('grantWCKBRole').addParam('private').addParam('to').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('grantRole', [hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes('MANAGER_ROLE')), taskArgs.to]),
    to: hre.ethers.utils.getAddress(wckb),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  // unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
  console.log(tx);
});

task('grantMirrorTokenRole').addParam('private').addParam('to').addParam('token').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('grantRole', [hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes('MANAGER_ROLE')), taskArgs.to]),
    to: hre.ethers.utils.getAddress(taskArgs.token),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('removeLimitTx').addParam('to').addParam('token').addParam('ckb').addParam('amount').addParam('limitSign').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  const amount = hre.ethers.BigNumber.from(taskArgs.from);
  const minWCKB = hre.ethers.BigNumber.from(taskArgs.ckb);
  let unsignedTx = {
    data: abi.encodeFunctionData('removeLimitTx', [taskArgs.token, amount, minWCKB, taskArgs.to, taskArgs.sign]),
    to: hre.ethers.utils.getAddress(crosschain),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('addWhitelist').addParam('token').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('addWhitelist', [taskArgs.token]),
    to: hre.ethers.utils.getAddress(crosschain),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount();
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('mintCKB').addParam('private').addParam('amount').addParam('to').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  console.log(signer.address);
  let unsignedTx = {
    data: abi.encodeFunctionData('mint', [taskArgs.to, hre.ethers.utils.parseUnits(taskArgs.amount, 8)]),
    to: hre.ethers.utils.getAddress(wckb),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('wckbBalance').addParam('account').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract(wckb, abi, signer);
  console.log(await contract.balanceOf(taskArgs.account));
});

task('hasRole').addParam('account').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract(wckb, abi, signer);
  const account = hre.ethers.utils.getAddress(taskArgs.account);
  console.log(await contract.hasRole(hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes('MANAGER_ROLE')), account));
});

task('setTokenConfig').addParam('token').addParam('threshold').addParam('fee').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('setTokenConfig', [taskArgs.token, [hre.ethers.BigNumber.from(taskArgs.fee), hre.ethers.BigNumber.from(taskArgs.threshold)]]),
    to: hre.ethers.utils.getAddress('0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2'),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('setWCKBMin').addParam('amount').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('setWCKBMin', [hre.ethers.utils.parseUnits(taskArgs.amount, 8)]),
    to: hre.ethers.utils.getAddress(crosschain),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('approveWCKB').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('approve', ['0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2', hre.ethers.utils.parseUnits('1000000', 18)]),
    to: hre.ethers.utils.getAddress(wckb),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('getTransaction').addParam('tx').setAction(async (taskArgs, hre) => {
  console.log(await hre.ethers.provider.getTransaction(taskArgs.tx));
});

task('checkTxHash').addParam('hash').setAction(async (args, hre) => {
  console.log('check tx hash:', args.hash);
  console.log(await hre.ethers.provider.getTransaction(args.hash));
})

task('generateWallet', '', async (taskArgs, hre) => {
  const wallet = hre.ethers.Wallet.createRandom();
  console.log(await wallet.getAddress());
  console.log(wallet.address);
  console.log(wallet.privateKey);
});

task('balance').addParam('account').setAction(async (args, hre) => {
  console.log(await hre.ethers.provider.getBalance(args.account));
});

task('transfer').addParam('private').addParam('to').setAction(async (args, hre) => {
  const wallet = new hre.ethers.Wallet(args.private, hre.ethers.provider);
  await wallet.sendTransaction({
    to: hre.ethers.utils.getAddress(args.to),
    value: hre.ethers.utils.parseEther('1.0'),
    nonce: 1,
  });
});

task('generateFromMnemonic').setAction(async (args, hre) => {
  const wallet = hre.ethers.Wallet.fromMnemonic('test test test test test test test test test test test junk');
  console.log(wallet.privateKey);
});

task('pubkey').addParam('private').setAction(async (args, hre) => {
  console.log(new hre.ethers.Wallet(args.private).address);
})

task('getLimitTxes', '', async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract(crosschain, abi, signer);
  console.log(await contract.limitTxes());
})

task('whitelist', '', async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract(crosschain, abi, signer);
  console.log(await contract.whitelist());
})

task('getWCKBMin', '', async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract(crosschain, abi, signer);
  console.log(await contract.getWCKBMin());
})

task('isWhitelist', '').addParam('token').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract(crosschain, abi, signer);
  console.log(taskArgs.token);
  console.log(await contract.isWhitelist(taskArgs.token));
})

// You need to export an object to set up your config
// Go to https://hardhat.org/config/ to learn more

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
  solidity: "0.8.20",
  networks: {
    axon: {
      url: process.env.AXON_URL|| "http://18.162.235.225:8000/",
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    axon_native: {
      url: 'http://127.0.0.1:8000',
      accounts: [],
    },
  },
};
