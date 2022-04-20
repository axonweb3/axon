import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

const pageName = "eth_getBlockTransactionCountByNumber.html";

let testDataInfo = null;
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getBlockTransactionCountByNumber", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
  * param1: real block number
  */
  it("eth_getBlockTransactionCountByNumber_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "0x1");
  });

  /**
 * param1: real decimal system block number
 */
  it("eth_getBlockTransactionCountByNumber_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.blockNumber.toString());
    await goto.check(page, "-32603");
  });

  /**
  * param1: the block number than axon current block number
  */
  it("eth_getBlockTransactionCountByNumber_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0xffffffff");
    await testType.type("1"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "0x0");
  });

  /**
  * param1: none
  */
  it("eth_getBlockTransactionCountByNumber_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "-32602");
  });

  /**
  * param1: real block number
  * param2: one more param
  */
  it("eth_getBlockTransactionCountByNumber_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type(testDataInfo.hexBlockNumber);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "0x1");
  });
});
