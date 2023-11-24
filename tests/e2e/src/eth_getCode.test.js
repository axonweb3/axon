import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getCode.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getCode", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
   * param1: real account address
   * param2: real number
   */
  it("eth_getCode_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "0x608060405234801561001057600080fd5b50600436106100b45760003560e01c80633e94dc0f116100715780633e94dc0f146102d057806370a082311461032857806395d89b4114610380578063a457c2d7146104035780");
  });

  /**
   * param1: real account address
   * param2: real number but not belong to the contract.
   */
  it.skip("eth_getCode_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x01");
    await goto.check(page, "0x");
  });

  /**
  * param1: real account address
  * param2: real number but not exist in the axon.
  */
  it("eth_getCode_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0xffffffff");
    await goto.check(page, "-32603");
  });

  /**
 * param1: real account address but not contract address
 * param2: real number
 */
  it("eth_getCode_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.hexBlockNumber);
    // Running eth_getCode_4 and eth_getCode_9 simultaneously causes axonweb3/axon/issues/1579
    // temporary solution: just wait 20 seconds after executing eth_getCode_4
    await goto.checkAndWait(page, "0x", 20);
  }, 25000);

  /**
  * param1: real account address but not exist
  * param2: real number
  */
  it.skip("eth_getCode_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x7f5659Aa997bC1473985f4391D7435f17C43b11z");
    await param2.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "0x");
  });

  /**
  * param1: real address
  * param2: real decimal numlber
  */
  it("eth_getCode_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x7f5659Aa997bC1473985f4391D7435f17C43b11f");
    await param2.type("1");
    await goto.check(page, "-32603");
  });

  /**
* param1: illegal address
* param2: real decimal numlber
*/
  it("eth_getCode_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1");// 0: none params 1: common params to request 2: more params
    await param1.type("0x7f5659Aa997bC147");
    await param2.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "-32603");
  });

  /**
  * param1: real address
  * param2: real decimal numlber
  * param3: more other param
  */
  it("eth_getCode_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("2");// 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "-32603");
  });

  /**
 * param1: real address
 */
  it("eth_getCode_9", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await goto.check(page, "-32603");
  });

  /**
 * none param
 */
  it("eth_getCode_10", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "-32603");
  });
});
