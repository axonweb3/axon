import goto from "./goto";

const pageName = "eth_gasPrice.html";
describe("eth_gasPrice", () => {
  /**
  * common real test
  */
  it("eth_gasPrice_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");
    await goto.check(page, "0x8");
  });

  /**
  * more params
  */
  it("eth_gasPrice_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "0x8");
  });
});
