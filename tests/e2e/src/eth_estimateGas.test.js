import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_estimateGas.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_estimateGas", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
  * param1: real account address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
    */
  it.skip("eth_estimateGas_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real account address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: error data
  * param7: real block number
  */
  it("eth_estimateGas_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06ddde03");
    await param7.type("latest");
    await goto.check(page, "0x");
  }, 100000);

  /**
  * param1: real account address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: value is 0x50
  * param6: real data
  * param7: real block number
   */
  it.skip("eth_estimateGas_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x50");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real account address
  * param2: real to address
  * param3: set gas is 0x0
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it.skip("eth_estimateGas_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0");
    await param4.type("0x8");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real account address
  * param2: none to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it("eth_estimateGas_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("");
    await param3.type("0x0200000000");
    await param4.type("0x8");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "-32603");
  }, 100000);

  /**
  * param1: real account address is empty
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it("eth_estimateGas_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("");
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x8");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "-32602");
  }, 100000);

  /**
  * param1: real address
  * param2: real to address
  * param3: real gas but more than system gaslimit
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it.skip("eth_estimateGas_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0xfffffffff");
    await param4.type("0x8");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice but is a big number
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it.skip("eth_estimateGas_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x210000");
    await param4.type("0xfffffffffff");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real address but is not exist in axon
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice but is a big number
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it.skip("eth_estimateGas_9", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x210000");
    await param4.type("0xfffffffffff");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real account address
  * param2: real to address but is not exist in axon
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it("eth_estimateGas_10", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x63010dD4c3164dd0D73eCB518972916161fBACd9");
    await param3.type("0x210000");
    await param4.type("0xfffffffffff");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x");
  }, 100000);

  /**
  * param1: real account address but is not exist in axon
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it.skip("eth_estimateGas_11", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x735EaC8A5f3F197799f2FEaEbc0F6B3F6e4c345D");
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x210000");
    await param4.type("0xfffffffffff");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
   * param1: illegal from address
   * param2: real to address
   * param3: real gas
   * param4: real gasPrice
   * param5: real value
   * param6: real data
   * param7: real block number
   */
  it("eth_estimateGas_12", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x735EaC8A5f3F19779");
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "-32603");
  }, 100000);

  /**
   * param1: real from address
   * param2: illegal to address
   * param3: real gas
   * param4: real gasPrice
   * param5: real value
   * param6: real data
   * param7: real block number
   */
  it("eth_estimateGas_13", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type("0xb484fd480E59862163");
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "-32602");
  }, 100000);

  /**
  * param1: real from address
  * param2: to address is none
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it("eth_estimateGas_14", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("4");
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x102330940AD4C6A8a0Fe35A");
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0xffffffffffffffff");
  }, 100000);
  /**
  * param1: from is none
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it.skip("eth_estimateGas_15", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("5");
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
    * param1: real from address
    * param2: real to address
    * param3: gas is none
    * param4: real gasPrice
    * param5: real value
    * param6: real data
    * param7: real block number
    */
  it.skip("eth_estimateGas_16", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("6");
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
    * param1: real from address
    * param2: real to address
    * param3: real gas
    * param4: gasPrice is none
    * param5: real value
    * param6: real data
    * param7: real block number
    */
  it.skip("eth_estimateGas_17", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("7");
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
    * param1: real from address
    * param2: real to address
    * param3: real gas
    * param4: real gasPrice
    * param5: value is none
    * param6: real data
    * param7: real block number
    */
  it.skip("eth_estimateGas_18", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("8");
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);

  /**
  * param1: real from address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: data is none
  * param7: real block number
  */
  it.skip("eth_estimateGas_19", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("9");
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x5f05");
  }, 100000);
  /**
  * param1: real from address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: block number is none
  */
  it("eth_estimateGas_20", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("10");
    await param1.type(testDataInfo.contractAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "-32602");
  }, 100000);

  /**
  * params: none
  */
  it("eth_estimateGas_21", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0");
    await goto.check(page, "-32602");
  }, 100000);

  /**
  * param1: real account address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  */
  it("eth_estimateGas_22", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("2");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "0x");
  }, 100000);

  /**
  * param1: real account address
  * param2: real to address
  * param3: real gas
  * param4: real gasPrice
  * param5: real value
  * param6: real data
  * param7: real block number
  * param8: one more param in the call request struct
  */
  it("eth_estimateGas_23", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    await testType.type("11");// 0: none params  1：common params to request   2: more params
    await param1.type(testDataInfo.accountAddress);
    await param2.type(testDataInfo.contractAddress);
    await param3.type("0x0200000000");
    await param4.type("0x08");
    await param5.type("0x0");
    await param6.type("0x06fdde03");
    await param7.type("latest");
    await goto.check(page, "-32602");
  }, 100000);
});
