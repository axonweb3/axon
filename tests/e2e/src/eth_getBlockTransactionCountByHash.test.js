import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

const pageName = "eth_getBlockTransactionCountByHash.html";

let testDataInfo = null;
describe("eth_getBlockTransactionCountByHash", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
  * param1: real block hash
  */
  it("eth_getBlockTransactionCountByHash_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.blockHash);
    await goto.check(page, "0x1");
  });

  /**
  * param1:non-existent in axon 0x34e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252874
  */
  it("eth_getBlockTransactionCountByHash_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x34e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252874");
    await testType.type("1"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "0x0");
  });

  /**
  * param1: 0x34e5f68936e20c7d2aef343
  */
  it("eth_getBlockTransactionCountByHash_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x34e5f68936e20c7d2aef343");
    await testType.type("1"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "-32602");
  });

  /**
  * param1: none
  */
  it("eth_getBlockTransactionCountByHash_4", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "-32602");
  });

  /**
  * param1: real block hash
  */
  it("eth_getBlockTransactionCountByHash_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type(testDataInfo.blockHash);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params 3: less params
    await goto.check(page, "0x1");
  });
});
