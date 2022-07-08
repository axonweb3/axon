const { task } = require("hardhat/config");

require("@nomiclabs/hardhat-waffle");
require('dotenv').config();

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
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('deployMirrorToken', 'deploy mirror token on axon').addParam('private').addParam('name').addParam('symbol').setAction(async (taskArgs, hre) => {
  const signer = new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  const MirrorToken = await hre.ethers.getContractFactory('MirrorToken');
  let unsignedTx = MirrorToken.getDeployTransaction(taskArgs.name, taskArgs.symbol);
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt);
});

task('crossAt', 'cross at').addParam('to').addParam('private').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('lockAT', [taskArgs.to]),
    to: hre.ethers.utils.getAddress('0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2'),
    value: hre.ethers.utils.parseEther('0.1'),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
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
    to: '0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2',
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
  const signedTx = await signer.signTransaction(unsignedTx);
  const tx = await hre.ethers.provider.sendTransaction(signedTx);
  const receipt = await tx.wait();
  console.log(receipt.logs.map((x) => {
    if (x.address == '0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2') {
      return abi.parseLog(x);
    }
  }))
  console.log(receipt);
});

task('grantWCKBRole').addParam('private').addParam('to').setAction(async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/MirrorToken.sol/MirrorToken.json').abi);
  const signer =new hre.ethers.Wallet(taskArgs.private, hre.ethers.provider);
  let unsignedTx = {
    data: abi.encodeFunctionData('grantRole', [hre.ethers.utils.keccak256(hre.ethers.utils.toUtf8Bytes('MANAGER_ROLE')), taskArgs.to]),
    to: hre.ethers.utils.getAddress('0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352'),
  };
  unsignedTx = await signer.populateTransaction(unsignedTx);
  unsignedTx.nonce = await signer.getTransactionCount() + 1;
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
    data: abi.encodeFunctionData('mint', [taskArgs.to, hre.ethers.utils.parseUnits(taskArgs.amount, 18)]),
    to: hre.ethers.utils.getAddress('0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352'),
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
  const contract = new hre.ethers.Contract('0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352', abi, signer);
  console.log(await contract.balanceOf(taskArgs.account));
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
    data: abi.encodeFunctionData('setWCKBMin', [hre.ethers.BigNumber.from(taskArgs.amount)]),
    to: hre.ethers.utils.getAddress('0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2'),
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
    data: abi.encodeFunctionData('approve', ['0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2', hre.ethers.BigNumber.from(100000000000)]),
    to: hre.ethers.utils.getAddress('0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352'),
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

task('getLimitTxes', '', async (taskArgs, hre) => {
  const abi = new hre.ethers.utils.Interface(require('./artifacts/contracts/crosschain.sol/CrossChain.json').abi);
  const signer = await hre.ethers.getSigner();
  const contract = new hre.ethers.Contract('0xF67Bc4E50d1df92b0E4C61794A4517AF6a995CB2', abi, signer);
  console.log(await contract.limitTxes());
})

// You need to export an object to set up your config
// Go to https://hardhat.org/config/ to learn more

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
  solidity: "0.8.4",
  networks: {
    axon: {
      url: process.env.AXON_URL|| "",
      accounts:
        process.env.PRIVATE_KEY !== undefined ? [process.env.PRIVATE_KEY] : [],
    },
    axon_native: {
      url: 'http://127.0.0.1:8000',
      accounts: [],
    },
  },
};
