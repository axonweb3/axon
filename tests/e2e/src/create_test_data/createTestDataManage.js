import Web3 from "web3";
import fs from "fs";

import Config from "../../config";

import erc20 from "./ERC20.json";

const basePath = "./src/test_data_temp_file";
const option = { timeout: 1000 * 30 };
const web3 = new Web3(new Web3.providers.HttpProvider(Config.getIns().axonRpc.url, option));

const transactionInfo = {
  contractAddress: "",
  transactionHash: "",
  blockNumber: "",
  hexBlockNumber: "",
  blockHash: "",
  transactionIndex: "",
  accountAddress: "",
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
    maxPriorityFeePerGas: 2500,
    maxFeePerGas: 2500,
    gasLimit: web3.utils.stringToHex("21000"),
    chainId: Config.getIns().axonRpc.chainId,
    data,
  };
  const transaction = await accountFrom.signTransaction(tx);
  return web3.eth.sendSignedTransaction(transaction.rawTransaction);
};

export async function createTransactionData() {
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
  transactionInfo.accountAddress = accountFrom.address;
  transactionInfo.hexBlockNumber = `0x${receipt.blockNumber.toString(16)}`;
  await savejson(`${basePath}/testData_1.json`, transactionInfo);
}

export function readTestDataAsJson(testFileName) {
  let data;
  try {
    const filePath = `${basePath}/${testFileName}`;
    const jsonData = fs.readFileSync(filePath, "binary").toString();
    data = JSON.parse(jsonData);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.log(err);
  }
  return data;
}

export async function resetTestTmpFiles() {
  try {
    fs.rmdirSync(basePath, { recursive: true });
    fs.mkdirSync(`${basePath}/`);
  } catch (ex) {
    fs.mkdirSync(`${basePath}/`);
  }
}
