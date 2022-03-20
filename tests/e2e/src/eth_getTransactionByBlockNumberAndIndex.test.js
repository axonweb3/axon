// eslint-disable-next-line
import { goto } from "./goto";
// eslint-disable-next-line import/named
import { testDataManage } from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getTransactionByBlockNumberAndIndex.html";
describe("eth_getTransactionByBlockNumberAndIndex", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
});

/**
* param1: the real block number
* param2: the real transaction index
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  const param2 = await page.$(goto.pageIds.param2Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type(`0x${testDataInfo.blockNumber.toString(16)}`);
  await param2.type(`0x${testDataInfo.transactionIndex.toString(16)}`);
  await goto.check(page, testDataInfo.blockHash);
});

/**
* param1: the real block number
* param2: error transaction index
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  const param2 = await page.$(goto.pageIds.param2Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type(`0x${testDataInfo.blockNumber.toString(16)}`);
  await param2.type("0xffff");
  await goto.check(page, "null");
});

/**
* param1: the not exist block number
* param2: real transaction index
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  const param2 = await page.$(goto.pageIds.param2Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type("0xfffffff");
  await param2.type(`0x${testDataInfo.transactionIndex.toString(16)}`);
  await goto.check(page, "null");
});

/**
* param1: Illegal block number
* param2: real transaction index
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  const param2 = await page.$(goto.pageIds.param2Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type("123");
  await param2.type(`0x${testDataInfo.transactionIndex.toString(16)}`);
  await goto.check(page, "-32602");
});

/**
* param1: real block number
* param2: none
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  await param1.type(`0x${testDataInfo.blockNumber.toString(16)}`);
  await testType.type("3"); // 0: none params 1: common params to request 2: more params
  await goto.check(page, "-32602");
});

/**
* param1: none
* param2: none
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  await testType.type("0"); // 0: none params 1: common params to request 2: more params
  await goto.check(page, "-32602");
});

/**
* param1: the real block number
* param2: the real number index
*/
it("eth_getTransactionByBlockNumberAndIndex_1", async () => {
  await goto.goto(page, pageName);
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  const param2 = await page.$(goto.pageIds.param2Id);
  await testType.type("2"); // 0: none params 1: common params to request 2: more params
  await param1.type(`0x${testDataInfo.blockNumber.toString(16)}`);
  await param2.type(`0x${testDataInfo.transactionIndex.toString(16)}`);
  await goto.check(page, "-32602");
});
