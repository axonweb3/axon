import goto from "./goto";

const pageName = "eth_blockNumber.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_blockNumber", () => {
  /**
   * real common request
   */
  it("eth_blockNumber_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x");
  });
  /**
  * param: one more param
  */
  it("eth_blockNumber_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x");
  });
});
