import goto from "./goto";

const pageName = "eth_submitWork.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_submitWork", () => {
  /**
   * param1: real nonce
   * param2: real pow hash
   * param3: real hex data
   */
  it.skip("eth_submitWork_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x1");
    await param2.type("0x18de99824561e6022e3b66797be8dd9f4d0310bceb15dec0e9e3fb18bebd2691");
    await param3.type("0x18de99824561e6022e3b66797be8dd9f4d0310bceb15dec0e9e3fb18bebd2691");
    await goto.check(page, "true");
  });
  /**
   * params: none
   */
  it.skip("eth_submitWork_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "null");
  });
  /**
   * param1: real nonce
   * param2: real pow hash
   * param3: real hex data
   * param4: one more param
   */
  it.skip("eth_submitWork_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    const param3 = await page.$(goto.pageIds.param3Id);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x1");
    await param2.type("0x18de99824561e6022e3b66797be8dd9f4d0310bceb15dec0e9e3fb18bebd2691");
    await param3.type("0x18de99824561e6022e3b66797be8dd9f4d0310bceb15dec0e9e3fb18bebd2691");
    await goto.check(page, "true");
  });
});
