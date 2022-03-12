// eslint-disable-next-line import/no-import-module-exports
import { Config } from "../../config";

const yaml = require("js-yaml");
const Web3 = require("web3");

const fs = require("fs");
// const EthereumTx = require('ethereumjs-tx').Transaction;
const erc20 = require("./ERC20.json");

const option = { timeout: 1000 * 30 };
const web3 = new Web3(new Web3.providers.HttpProvider(Config.getIns().axonRpc.url, option));

const transactionInfo = {
  contractAddress: "",
  transactionHash: "",
  blockNumber: "",
  blockHash: "",
  transactionIndex: "",
};
const accountFrom = web3.eth.accounts.privateKeyToAccount(Config.getIns().hexPrivateKey);
const sendTransaction = async (account, data) => {
  const nonce = (await web3.eth.getTransactionCount(accountFrom.address)) + 1;
  const tx = {
    type: 2,
    nonce,
    maxPriorityFeePerGas: 250,
    maxFeePerGas: 250,
    gasLimit: web3.utils.stringToHex("21000"),
    chainId: Config.getIns().axonRpc.chainId,
    data,
  };
  const transaction = await accountFrom.signTransaction(tx);
  return web3.eth.sendSignedTransaction(transaction.rawTransaction);
};

const createTransactionData = async () => {
  const contract = new web3.eth.Contract(erc20.abi);
  const txOptions = { data: erc20.bytecode, arguments: ["TT", "TTT"] };
  const data = contract.deploy(txOptions).encodeABI();
  const receipt = await sendTransaction(
    accountFrom.address,
    data,
  );
  transactionInfo.contractAddress = receipt.contractAddress;
  transactionInfo.transactionHash = receipt.transactionHash;
  transactionInfo.blockHash = receipt.blockHash;
  transactionInfo.blockNumber = receipt.blockNumber;
  transactionInfo.transactionIndex = receipt.transactionIndex;
  const yamlStr = yaml.dump(transactionInfo);
  fs.writeFileSync("./src/create_test_data/testData.yaml", yamlStr, "utf8");
};
module.exports.createTestData = { createTransactionData };
