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
  it("eth_sendRawTransaction_1", async () => {
    const tx = await createTransactionData.sendRawTestTx();
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(tx);
    await goto.check(page, "0x");
  });

  /**
   * real common request,common transaction
   */
  it("eth_sendRawTransaction_2", async () => {
    const tx = await createTransactionData.sendPreEip155RawTestTx();
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type(tx);
    await goto.check(page, "0x");
  });

  /**
  * param: one more param
  */
  it("eth_sendRawTransaction_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "32603");
  });
});
