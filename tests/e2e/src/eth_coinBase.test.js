// eslint-disable-next-line
import { goto } from "./goto";
const pageName = "eth_coinBase.html";
describe("eth_coinBase", () => {
  /**
  * common real test
  * axon haven't wallet now. so skip thistest case
  */
  it("eth_coinBase_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");
    await goto.check(page, "null");
  });

  /**
  * more params
  */
  it("eth_coinBase_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");// 0: none params  1：common params to request   2: more params
    await goto.check(page, "null");
  });
});
