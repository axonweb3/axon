import goto from "./goto";

const pageName = "eth_newBlockFilter.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_newBlockFilter", () => {
  /**
   * none param
   */
  it("eth_newBlockFilter_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x");
  });
  /**
  * param: one more param
  */
  it("eth_newBlockFilter_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x");
  });
});
