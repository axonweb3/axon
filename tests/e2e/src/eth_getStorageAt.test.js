import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getStorageAt.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getStorageAt", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");

  /**
  * param1: real address
  * param2: 0x0
  * param3: real block number
  */
  it.skip("eth_getStorageAt_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "0x");
  });

  /**
  * param1: real address
  * param2: 0x0
  * param3: real block number
  */
  it.skip("eth_getStorageAt_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x02");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    // await goto.check(page, "0x0422ca8b0a00a425000000");
    await goto.check(page, "0x");
  });

  /**
  * param1: not exist address
  * param2: 0x0
  * param3: real block number
  */
  it.skip("eth_getStorageAt_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x295a70b2de5e3953354a6a8344e616ed314d7252");
    await param2.type("0x02");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "Can't find this address");
  });

  /**
  * param1: real address
  * param2: 0xfffff, that is not exist position
  * param3: latest
  */
  it.skip("eth_getStorageAt_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0xffffff");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "Can't find this position");
  });

  /**
  * param1: real address
  * param2: 0x0
  * param3: 0xfffffffff, that is not exist block number
  */
  it("eth_getStorageAt_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x02");
    await param3.type("0xffffffffff");
    await goto.check(page, "Can't find this block");
  });

  /**
  * param1: real address
  * param2: 0x0
  * param3: none
  */
  it.skip("eth_getStorageAt_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("3");// 0: none params  1: common params to request   2: more parameter 3:less params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x02");
    await goto.check(page, "-32603");
  });

  /**
  * none param
  */
  it("eth_getStorageAt_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "-32602");
  });
  /**
 * param1: Illegal address
 * param2: 0x0
 * param3: real block number
 */
  it("eth_getStorageAt_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x295a70b2de5e3953354a6a");
    await param2.type("0x02");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "invalid length 22");
  });

  /**
 * param1: Illegal address
 * param2: 0x0
 * param3: real block number
 */
  it.skip("eth_getStorageAt_9", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x05");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "0x0000000000000000000000000000000000000000000000000000000000000012");
  });

  /**
  * param1: real address
  * param2: 0x0
  * param3: real block number
  */
  it("eth_getStorageAt_10", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x02");
    await param3.type(`{ "blockNumber": "0x${testDataInfo.blockNumber.toString(16)}" }`);
    // await goto.check(page, "0x0422ca8b0a00a425000000");
    await goto.check(page, "0x");
  });

  /**
  * param1: real address
  * param2: 0x0
  * param3: real block hash
  */
  it("eth_getStorageAt_11", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x02");
    await param3.type(`{ "blockHash": "${testDataInfo.blockHash}" }`);
    // await goto.check(page, "0x0422ca8b0a00a425000000");
    await goto.check(page, "0x");
  });
});
