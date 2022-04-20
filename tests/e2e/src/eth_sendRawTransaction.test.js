import goto from "./goto";
import createTransactionData from "./create_test_data/createTestDataManage";

const pageName = "eth_sendRawTransaction.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_sendRawTransaction", () => {
  /**
   * real common request,common transaction
   */
  it.skip("eth_sendRawTransaction_1", async () => {
    const tx = await createTransactionData.sendRawTestTx();
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(tx);
    await goto.check(page, "true");
  });

  /**
  * param: one more param
  */
  it("eth_sendRawTransaction_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "true");
  });
});
