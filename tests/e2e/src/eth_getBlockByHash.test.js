import goto from "./goto";
import Config from "../config";

const pageName = "eth_getBlockByHash.html";
describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: Config.getIns().axonRpc.netWorkName,
      rpc: Config.getIns().axonRpc.url,
      chainId: Config.getIns().axonRpc.chainId,
    });
  });
  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  * param2:true
  */
  test("eth_getBlockByHash_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await param2.type("true");
    await goto.check(page, "max_priority_fee_per_gas");
  });

  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  * param2:false
  */
  test("eth_getBlockByHash_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await param2.type("false");
    await goto.check(page, "\"parentHash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\"");
  });

  /**
  * param1(non-existent in axon):0x34e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252874
  * param2:false
  */
  test("eth_getBlockByHash_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await param1.type("0x34e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252874");
    await param2.type("false");
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "null");
  });

  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  * param2:none
  */
  test("eth_getBlockByHash_4", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("3");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await goto.check(page, "-32602");
  });

  /**
  * param1:none
  * param2:none
  */
  test("eth_getBlockByHash_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "-32602");
  });

  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  * param2:true
  */
  test("eth_getBlockByHash_6", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await param2.type("true");
    await testType.type("2");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "-32602");
  });

  /**
  * param1:0x937373642bc
  * param2:true
  */
  test("eth_getBlockByHash_7", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await param1.type("0x937373642bc5ea582");
    await param2.type("true");
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "-32602");
  });
});
