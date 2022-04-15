import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getTransactionByHash.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getTransactionByHash", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
});

/**
* param1: the real tx hash
*/
it("eth_getTransactionByHash_1", async () => {
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type(testDataInfo.transactionHash);
  await goto.check(page, testDataInfo.blockHash);
});

/**
* param1: the not exist tx hash
*/
it("eth_getTransactionByHash_2", async () => {
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type("0x5bbb63f07062d74bedef3c34b46991b9d519dca93438200981371b679d9c4c76");
  await goto.check(page, "null");
});

/**
* param1: Illegal tx hash
*/
it("eth_getTransactionByHash_3", async () => {
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  await testType.type("1"); // 0: none params 1: common params to request 2: more params
  await param1.type("0x5bbb63f07062d74bedef3c34b46991b9");
  await goto.check(page, "-32602");
});

/**
* param1: none
*/
it("eth_getTransactionByHash_4", async () => {
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  await testType.type("0"); // 0: none params 1: common params to request 2: more params
  await param1.type(testDataInfo.transactionHash);
  await goto.check(page, "-32602");
});

/**
* param1: real tx hash
* param2: one more param
*/
it("eth_getTransactionByHash_5", async () => {
  const testType = await page.$(goto.pageIds.testTypeId);
  const param1 = await page.$(goto.pageIds.param1Id);
  await param1.type(testDataInfo.transactionHash);
  await testType.type("2"); // 0: none params 1: common params to request 2: more params
  await goto.check(page, testDataInfo.blockHash);
});
