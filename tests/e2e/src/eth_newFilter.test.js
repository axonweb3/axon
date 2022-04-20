// eslint-disable-next-line
import goto from "./goto";
import testDataManage from "./create_test_data/createTestDataManage";
import Config from "../config";

const pageName = "eth_newFilter.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_newFilter", () => {
  const testDataInfo = testDataManage.readTestDataAsJson("testData_1.json");
  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real a address in address array
  * param5: topics is [], that any topics
  */
  it.skip("eth_newFilter_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    // const param1 = await page.$(goto.pageIds.param1Id);
    // const param2 = await page.$(goto.pageIds.param2Id);
    // const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("4"); // topics is [], that any topics
    // await param1.type("0x1");
    // await param2.type("0x2");
    // await param3.type(testDataInfo.accountAddress);
    await goto.check(page, "null");
  }, 100000);
  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real a address in address array
* param5: topics is [A]
*/
  it.skip("eth_newFilter_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    await testType.type("5"); // topics is [A], that any topics
    await param1.type("0x01");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.accountAddress);
    await param5.type(testDataInfo.topic1);
    await goto.check(page, "null");
  });
  /**
  * param1: real fromBlock
  * param2: real toBlock
  * param3: real a address in address array
  * param5: real topic1
  * param6: real topic2
  */
  it.skip("eth_newFilter_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    await testType.type("6"); // topics is [A,B]
    await param1.type("0x01");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.accountAddress);
    await param5.type(testDataInfo.topic1);
    await param6.type(testDataInfo.topic2);
    await goto.check(page, "null");
  });
  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real a address in address array
* param5: real topic1
* param6: real topic2
* param7: real topic1
* param8: real topic2
*/
  it.skip("eth_newFilter_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    const param8 = await page.$(goto.pageIds.param8Id);
    await testType.type("7"); // topics is [[A,B],[A,B]]
    await param1.type("0x01");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.accountAddress);
    await param5.type(testDataInfo.topic1);
    await param6.type(testDataInfo.topic2);
    await param7.type(testDataInfo.topic1);
    await param8.type(testDataInfo.topic2);
    await goto.check(page, "null");
  });

  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real a address in address array
* param4: real another address in address array
*/
  it.skip("eth_newFilter_5", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param4 = await page.$(goto.pageIds.param4Id);
    await testType.type("8"); // topics is []
    await param1.type("0x01");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.accountAddress);
    await param4.type(Config.getIns().account2);
    await goto.check(page, "null");
  });

  /**
 * param1: real fromBlock
 * param2: real toBlock that is not in axon.
 * param3: real a address in address array
 */
  it.skip("eth_newFilter_6", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("9"); // topics is []
    await param1.type("0x01");
    await param2.type("0xffffffffff");
    await param3.type(testDataInfo.accountAddress);
    await goto.check(page, "null");
  });

  /**
 * param1: real fromBlock that is not in axon
 * param2: real toBlock
 * param3: real a address in address array
 */
  it.skip("eth_newFilter_7", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("9"); // topics is []
    await param1.type("0xffffffffff");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.accountAddress);
    await goto.check(page, "null");
  });

  /**
* param1: real fromBlock
* param2: real toBlock
*/
  it.skip("eth_newFilter_8", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("10"); // topics is []
    await param1.type("0x1");
    await param2.type(testDataInfo.hexBlockNumber);
    await goto.check(page, "null");
  });
  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real a address in address array
* param5: real topic1
* param6: real topic2
* param7: real topic1
* param8: real topic2
*/
  it.skip("eth_newFilter_9", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    const param8 = await page.$(goto.pageIds.param8Id);
    await testType.type("2"); // topics is [[A,B],[A,B]]
    await param1.type("0x01");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type(testDataInfo.accountAddress);
    await param5.type(testDataInfo.topic1);
    await param6.type(testDataInfo.topic2);
    await param7.type(testDataInfo.topic1);
    await param8.type(testDataInfo.topic2);
    await goto.check(page, "null");
  });
  /**
* param1: real fromBlock
* param2: real toBlock
* param3: real a address in address array
* param5: real topic1
* param6: real topic2
* param7: real topic1
* param8: real topic2
*/
  it.skip("eth_newFilter_10", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    const param5 = await page.$(goto.pageIds.param5Id);
    const param6 = await page.$(goto.pageIds.param6Id);
    const param7 = await page.$(goto.pageIds.param7Id);
    const param8 = await page.$(goto.pageIds.param8Id);
    await testType.type("2"); // topics is [[A,B],[A,B]]
    await param1.type("0x01");
    await param2.type(testDataInfo.hexBlockNumber);
    await param3.type("0x8ab0CF264DF99D83525e9E1");
    await param5.type(testDataInfo.topic1);
    await param6.type(testDataInfo.topic2);
    await param7.type(testDataInfo.topic7);
    await param8.type(testDataInfo.topic8);
    await goto.check(page, "null");
  });
  /**
  * none params
  */
  it.skip("eth_newFilter_11", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("3"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x0");
  });
});
