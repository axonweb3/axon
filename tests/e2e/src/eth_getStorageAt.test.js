// eslint-disable-next-line
import { goto } from "./goto";
import { Config } from "../config";
// eslint-disable-next-line import/named
import { testDataManage } from "./create_test_data/createTestDataManage";

let testDataInfo = null;
const pageName = "eth_getStorageAt.html";
describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: Config.getIns().axonRpc.netWorkName,
      rpc: Config.getIns().axonRpc.url,
      chainId: Config.getIns().axonRpc.chainId,
    });
    testDataInfo = await testDataManage.readTestDataAsJson("testData_1.json");
  });
  /**
  * param1:real address
  * param2:0x0
  * param3:real block number
  */
  test("eth_getStorageAt_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x0");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "TT");
  });

  /**
  * param1:not exist address
  * param2:0x0
  * param3:real block number
  */
  test("eth_getStorageAt_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x295a70b2de5e3953354a6a8344e616ed314d7252");
    await param2.type("0x0");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "Can't find this address");
  });

  /**
  * param1:real address
  * param2:0xfffff, that is not exist position
  * param3:latest
  */
  test("eth_getStorageAt_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0xfffff");
    await param3.type(`0x${testDataInfo.blockNumber.toString(16)}`);
    await goto.check(page, "Can't find this position");
  });

  /**
  * param1:real address
  * param2:0x0
  * param3:0xfffffffff, that is not exist block number
  */
  test("eth_getStorageAt_4", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x0");
    await param3.type("0xfffffffff");
    await goto.check(page, "Can't find this block");
  });

  /**
  * param1:real address
  * param2:0x0
  * param3:none
  */
  test("eth_getStorageAt_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("3");// 0: none params  1：common params to request   2: more parameter 3:less parameter
    await param1.type(testDataInfo.contractAddress);
    await param2.type("0x0");
    await goto.check(page, "-32603");
  });

  /**
  * none param
  */
  test("eth_getStorageAt_6", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0");// 0: none params  1：common params to request   2: more parameter
    await goto.check(page, "incorrect number of arguments");
  });
});
