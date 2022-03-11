import { goto } from "./goto";

const pageName = "eth_getStorageAt.html";
describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: goto.axonRpc.netWrokName,
      rpc: goto.axonRpc.url,
      chainId: goto.axonRpc.chianId,
    });
  });
  /**
  * param1:0x295a70b2de5e3953354a6a8344e616ed314d7253 // fixme
  * param2:0x0 // fixme
  * param3:latest // fixme
  */
  test("eth_getStorageAt_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x295a70b2de5e3953354a6a8344e616ed314d7253");
    await param2.type("0x0");
    await param3.type("latest");
    await goto.check(page, "0x0");
  });

  /**
  * param1:0x295a70b2de5e3953354a6a8344e616ed314d7252
  * param2:0x0
  * param3:latest
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
    await param3.type("latest");
    await goto.check(page, "0x0");
  });

  /**
  * param1:0x95a70b2de5e3953354a6a8344e616ed314d7252
  * param2:0x0
  * param3:latest
  */
  test("eth_getStorageAt_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x95a70b2de5e3953354a6a8344e616ed314d7252");
    await param2.type("0x0");
    await param3.type("latest");
    await goto.check(page, "0x0");
  });

  /**
  * param1:0x8ab0CF264DF99D83525e9E11c7e4db01558AE1b1
  * param2:0x0
  * param3:0xfffffffff
  */
  test("eth_getStorageAt_4", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x8ab0CF264DF99D83525e9E11c7e4db01558AE1b1");
    await param2.type("0x0");
    await param3.type("0xfffffffff");
    await goto.check(page, "Can't find this block");
  });

  /**
  * param1:0x8ab0CF264DF99D83525e9E11c7e4db01558AE1b1
  * param2:0x0
  * param3:none
  */
  test("eth_getStorageAt_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("3");// 0: none params  1：common params to request   2: more parameter 3:less parameter
    await param1.type("0x8ab0CF264DF99D83525e9E11c7e4db01558AE1b1");
    await param2.type("0x0");
    await goto.check(page, "Ox0");
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
