// eslint-disable-next-line import/no-import-module-exports
import { Config } from "../../config";

const Web3 = require("web3");
const iconv = require("iconv-lite");
const fs = require("fs");

const erc20 = require("./ERC20.json");

const basePath = "./src/test_data_temp_file";
const option = { timeout: 1000 * 30 };
const web3 = new Web3(new Web3.providers.HttpProvider(Config.getIns().axonRpc.url, option));

const transactionInfo = {
  contractAddress: "",
  transactionHash: "",
  blockNumber: "",
  blockHash: "",
  transactionIndex: "",
};

const savejson = async (filePath, data) => {
  const dataStr = JSON.stringify(data, null, 4);
  if (dataStr) {
    try {
      fs.writeFileSync(filePath, dataStr);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.log(`save error ${err}`);
    }
  }
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
  await savejson(`${basePath}/testData_1.json`, transactionInfo);
};

const readTestDataAsJson = (testFileName) => {
  let data;
  try {
    const filePath = `${basePath}/${testFileName}`;
    const jsonData = iconv.decode(fs.readFileSync(filePath, "binary"), "utf8").toString();
    data = JSON.parse(jsonData);
    // eslint-disable-next-line no-console
    console.log(data);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.log(err);
  }
  return data;
};

const resetTestTmpFiles = async () => {
  try {
    fs.rmdirSync(basePath, { recursive: true });
    fs.mkdirSync(`${basePath}/`);
  } catch (ex) {
    fs.mkdirSync(`${basePath}/`);
  }
};

module.exports.testDataManage = { createTransactionData, readTestDataAsJson, resetTestTmpFiles };
