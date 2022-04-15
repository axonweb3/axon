import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getLogs.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getLogs", () => {
  testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real address
  * param4: real topic1
  * param5: real topic2
  * param6: real blockHash
  */
  it.skip("eth_getLogs_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);

  /**
* param1: fromBlock is none
* param2: toBlock is none
* param3: address is none
* param4: topic1 is none
* param5: topic2 is nont
* param6: blockHash is none
*/
  it.skip("eth_getLogs_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("4");// 0: none params  1：common params to request   2: more params
    await goto.check(page, "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
  }, 100000);

  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real address
* param4: topic1 is none
* param5: topic2 is none
* param6: real blockHash
*/
  it.skip("eth_getLogs_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("5");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);

  /**
* param1: real fromBlock
* param2: real toBlock
* param3: address is none
* param4: real topic1
* param5: real topic2
* param6: real blockHash
*/
  it.skip("eth_getLogs_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("6");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);
  /**
* param1: real fromBlock
* param2: toBlock is none
* param3: real address
* param4: real topic1
* param5: real topic2
* param6: real blockHash
*/
  it.skip("eth_getLogs_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("7");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);

  /**
* param1: fromBlock is none
* param2: real toBlock
* param3: real address
* param4: real topic1
* param5: real topic2
* param6: real blockHash
*/
  it.skip("eth_getLogs_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("8");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);

  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real address
* param4: real topic1
* param5: real topic2
* param6: blockHash is none
*/
  it.skip("eth_getLogs_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("9");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);

  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real address
* param4: real topic1
* param5: real topic2
* param6: real none
*/
  it.skip("eth_getLogs_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("10");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);
  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real none
* param4: topic1 is none
* param5: topic2 is none
* param6: real none
*/
  it.skip("eth_getLogs_9", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("11");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);
  /**
* param1: real fromBlock
* param2: real none
* param3: real none
* param4: topic1 is none
* param5: topic2 is none
* param6: real none
*/
  it.skip("eth_getLogs_10", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("12");
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);
  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real address
  * param4: real topics,set a  topic which is not exist in axon.
  * param5: topic2 is none
  * param6: real blockHash
  */
  it.skip("eth_getLogs_11", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type("0x7be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e7");
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "[]");
  }, 100000);

  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real address
  * param4: real topics,set a  error topic
  * param5: topic2 is none
  * param6: real blockHash
  */
  it.skip("eth_getLogs_12", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type("0x7be0079c531659141b4186f6b6457e7");
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "[]");
  }, 100000);

  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real address but is not in axon
  * param4: real topics
  * param5: topic2 is none
  * param6: real blockHash
  */
  it.skip("eth_getLogs_13", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type("0xd43d960B4Bb56CA1dA28a2B59D4c3f5dD28330aF");
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "[]");
  }, 100000);

  /**
  * param1: real fromBlock
  * param2: error toBlock
  * param3: real address
  * param4: real topics
  * param5: topic2 is none
  * param6: real blockHash
  */
  it.skip("eth_getLogs_14", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0x0");
    await param2.type("12321");
    await param3.type("0xd43d960B4Bb56CA1dA28a2B59D4c3f5dD28330aF");
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "[]");
  }, 100000);
  /**
* param1: error fromBlock
* param2: real toBlock
* param3: real address
* param4: real topics
* param5: topic2 is none
* param6: real blockHash
*/
  it.skip("eth_getLogs_15", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more params
    await param1.type("0");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type("0xd43d960B4Bb56CA1dA28a2B59D4c3f5dD28330aF");
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "[]");
  }, 100000);

  /**
* params: none
*/
  it.skip("eth_getLogs_16", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0");// 0: none params  1：common params to request   2: more params
    await goto.check(page, "[]");
  }, 100000);

  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real address
  * param4: real topic1
  * param5: real topic2
  * param6: real blockHash
  */
  it.skip("eth_getLogs_17", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("2");// 0: none params  1：common params to request   2: more params
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.contractAddress);
    await param4.type(testDataInfo.topic1);
    await param5.type(testDataInfo.topic2);
    await param6.type(testDataInfo.blockHash);
    await goto.check(page, "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0");
  }, 100000);
});
