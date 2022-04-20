import goto from "./goto";

const pageName = "eth_getFilterChange.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_getFilterChange", () => {
  /**
   * param1: real filterId
   */
  it.skip("eth_getFilterChange_1", async () => {
    await page.click("#btn_filter");
    await page.waitForFunction(
      (id) => document.getElementById(id).innerText !== "",
      {},
      "ret_filter_Id",
    );
    const request = page.$eval("#ret_filter_Id", (e) => e.innerText);
    await expect(request).resolves.not.toThrow();
    const filterId = await request;
    // eslint-disable-next-line no-console
    console.log("the filter id is : ", filterId.toString(10));
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1");
    await param1.type("0x1");
    await goto.check(page, "true");
  });

  /**
   * param1: filterId not in axon
   */
  it.skip("eth_getFilterChange_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1");
    await param1.type("0xffffff");
    await goto.check(page, "false");
  });

  /**
 * param1: real filterId
 * param2: one more param
 */
  it.skip("eth_getFilterChange_3", async () => {
    await goto.goto(page, pageName);
    await page.click("#btn_filter");
    await page.waitForFunction(
      (id) => document.getElementById(id).innerText !== "",
      {},
      "ret_filter_Id",
    );
    const request = page.$eval("#ret_filter_Id", (e) => e.innerText);
    await expect(request).resolves.not.toThrow();
    const filterId = await request;
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("2");
    await param1.type(filterId);
    await goto.check(page, "false");
  });

  /**
  * param: none
  */
  it.skip("eth_getFilterChange_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("3"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "false");
  });
});
