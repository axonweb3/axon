import goto from "./goto";

const pageName = "eth_hasHrate.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_hasHrate", () => {
  /**
  * common real test
  */
  it("eth_hasHrate_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");
    await goto.check(page, "0x1");
  });

  /**
  * more params
  */
  it("eth_hasHrate_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x1");
  });
});
