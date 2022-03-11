import { goto } from "./goto";

const pageName = "eth_getBlockTransactionCountByHash.html";
describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: goto.axonRpc.netWrokName,
      rpc: goto.axonRpc.url,
      chainId: goto.axonRpc.chianId,
    });
  });
  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  */
  test("eth_getBlockTransactionCountByHash_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await goto.check(page, "0x1");
  });

  /**
  * param1(non-existent in axon):0x34e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252874
  */
  test("eth_getBlockTransactionCountByHash_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x34e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252874");
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "0x0");
  });

  /**
  * param1:0x34e5f68936e20c7d2aef343
  */
  test("eth_getBlockTransactionCountByHash_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x34e5f68936e20c7d2aef343");
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "-32602");
  });

  /**
  * param1:none
  */
  test("eth_getBlockTransactionCountByHash_4", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "-32602");
  });

  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  */
  test("eth_getBlockTransactionCountByHash_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await testType.type("2");// 0: none params  1：common params to request   2: more parameter 3: less parameter
    await goto.check(page, "0x1");
  });
});
