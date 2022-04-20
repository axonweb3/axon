import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getTransactionReceipt.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getTransactionReceipt", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
   * param1: real tx hash
   */
  it("eth_getTransactionReceipt_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.transactionHash);
    await goto.check(page, testDataInfo.transactionHash);
  });
  /**
   * param1: tx hash is not in axon
   */
  it("eth_getTransactionReceipt_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x18de99824561e6022e3b66797be8dd9f4d0310bceb15dec0e9e3fb18bebd2692");
    await goto.check(page, "null");
  });
  /**
   * param1: none
   */
  it("eth_getTransactionReceipt_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "-32603");
  });
  /**
   * param1: real tx hash
   * param2: one more param
   */
  it("eth_getTransactionReceipt_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.transactionHash);
    await goto.check(page, "null");
  });
  /**
   * param1: illegal tx hash
   */
  it("eth_getTransactionReceipt_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x18de99824561e6022e3b66797be8dd9f4d0310b");
    await goto.check(page, "-32602");
  });
});
