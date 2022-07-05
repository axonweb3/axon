import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getTransactionCount.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getTransactionCount", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  // /**
  //  * param1: real account address
  //  * param2: real latest number
  //  */
  // it("eth_getTransactionCount_1", async () => {
  //   const testType = await page.$(goto.pageIds.testTypeId);
  //   const param1 = await page.$(goto.pageIds.param1Id);
  //   const param2 = await page.$(goto.pageIds.param2Id);
  //   await testType.type("1"); // 0: none params 1: common params to request 2: more params
  //   await param1.type(testDataInfo.accountAddress);
  //   await param2.type("latest");
  //   await goto.check(page, "0x4");
  // });

  /**
   * param1: set a contract address
   * param2: real block number: latest
   */
  it("eth_getTransactionCount_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("latest");
    await goto.check(page, "0x2");
  });

  /**
 * param1: the address is not in axon 0x9EeE8678045C389005EDe68D9A2a548f36342184
 * param2: real block number: latest
 */
  it("eth_getTransactionCount_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x9EeE8678045C389005EDe68D9A2a548f36342184");
    await param2.type("latest");
    await goto.check(page, "0x1");
  });

  /**
* param1: real address
* param2: the block number is not in axon
*/
  it("eth_getTransactionCount_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0xfffff");
    await goto.check(page, "-32001");
  });

  /**
  * param1: real address
  * param2: set decimal data without 0x to block number param
  */
  it("eth_getTransactionCount_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("1");
    await goto.check(page, "-32602");
  });

  /**
  * param1: illegal address
  * param2: real address
  */
  it("eth_getTransactionCount_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params  1: common params to request 2: more params
    await param1.type("0x8ab0CF264DF99D8");
    await param2.type("1");
    await goto.check(page, "-32602");
  });

  /**
 * param1: real address
 * param2: real address
 * param3: more param
 */
  it("eth_getTransactionCount_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("latest");
    await goto.check(page, "-32603");
  });

  /**
* param1: none
* param2: none
*/
  it("eth_getTransactionCount_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "-32603");
  });

  /**
 * param1:real address
 */
  it("eth_getTransactionCount_9", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("3"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await goto.check(page, "-32603");
  });
});
