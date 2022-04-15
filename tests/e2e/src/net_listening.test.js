import goto from "./goto";

const pageName = "net_listening.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("net_listening", () => {
  /**
   * real common request
   */
  it("net_listening_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "true");
  });
  /**
  * param: one more param
  */
  it("net_listening_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "true");
  });
});
