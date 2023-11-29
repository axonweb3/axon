import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getBalance.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getBalance", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
   * param1: real account address
   * param2: real other number
   */
  it.skip("eth_getBalance_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("0x0");
    await goto.check(page, "0x1bc16d775099bf800");
  });

  /**
   * param1: real account address
   * param2: real block number: 0x2
   */
  it.skip("eth_getBalance_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("0x2");
    await goto.check(page, "0x1bc16d775099bf800");
  });

  /**
   * param1: real account address
   * param2: real other number: latest
   */
  it("eth_getBalance_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("latest");
    await goto.check(page, "0x4ee2d6d415b77cb79cd725a1f28");
  });

  /**
   * param1: real account address
   * param2: none exist block number: 0xfffffff
   */
  it("eth_getBalance_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("0xfffffff");
    await goto.check(page, "0x0");
  });

  /**
   * param1: the address of contract address
   * param2: real block number
   */
  it("eth_getBalance_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("latest");
    await goto.check(page, "0x0");
  });

  /**
   * param1: the address but is not exsit in axon: 0x3dF82e36a8b27CE05815f88a8021b61aAbeF8B31
   * param2: real block number
   */
  it("eth_getBalance_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x3dF82e36a8b27CE05815f88a8021b61aAbeF8B31");
    await param2.type("latest");
    await goto.check(page, "0x0");
  });

  /**
   * param1: real address
   * param2: the block number is decimal format
   */
  it("eth_getBalance_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x3dF82e36a8b27CE05815f88a8021b61aAbeF8B31");
    await param2.type("1");
    await goto.check(page, "-32603");
  });

  /**
   * param1: illegal address
   * param2: real block number
   */
  it("eth_getBalance_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x3dF82e36a8b27CE05815f");
    await param2.type("1");
    await goto.check(page, "-32603");
  });

  /**
   * param1: real account address
   * param2: real other number
   * param3: more param
   */
  it("eth_getBalance_9", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("0x0");
    await goto.check(page, "-32603");
  });

  /**
   * param1: real account address
   */
  it("eth_getBalance_10", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("3"); // 0: none params  1: common params to request 2: more params
    await param1.type(testDataInfo.accountAddress);
    await goto.check(page, "-32602");
  });

  /**
   * param1: real account address
   */
  it("eth_getBalance_11", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params  1: common params to request 2: more params
    await goto.check(page, "-32602");
  });
});
