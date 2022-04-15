import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getBlockByNumber.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getBlockByNumber", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
  * param1: real block numer
  * param2: true
  */
  it("eth_getBlockByNumber_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.hexBlockNumber);
    await param2.type("true");
    await goto.check(page, "max_priority_fee_per_gas");
  });

  /**
  * param1: real block number
  * param2: false
  */
  it("eth_getBlockByNumber_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.hexBlockNumber);
    await param2.type("false");
    await goto.check(page, "parentHash");
  });

  /**
  * param1: non-existent block number in axon 0xfffffff
  * param2: true
  */
  it("eth_getBlockByNumber_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await param1.type("0xfffffff");
    await param2.type("true");
    await testType.type("1"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "null");
  });

  /**
  * param1: real block number
  * param2: none
  */
  it("eth_getBlockByNumber_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("3"); // 0: none params 1: common params to request 2: more params 3: less params
    await param1.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "-32602");
  });

  /**
  * param1: none
  * param2: none
  */
  it("eth_getBlockByNumber_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "-32602");
  });

  /**
  * param1: real block number
  * param2: true
  * param3: one more param
  */
  it("eth_getBlockByNumber_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await param1.type(testDataInfo.hexBlockNumber);
    await param2.type("true");
    await testType.type("2"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "-32602");
  });
});
