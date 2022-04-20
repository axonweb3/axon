import goto from "./goto";

const pageName = "eth_submitHashrate.html";
beforeEach(async () => {
  await goto.goto(page, pageName);
});
describe("eth_submitHashrate", () => {
  /**
   * param1: real hash rate
   * param2: real clinet id
   */
  it("eth_submitHashrate_1", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x0000000000000000000000000000000000000000000000000000000000500000");
    await param2.type("0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c");
    await goto.check(page, "true");
  });
  /**
   * param1: real hash rate
   * param2: real clinet id
   * param3: one more param
   */
  it("eth_submitHashrate_2", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("2"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x0000000000000000000000000000000000000000000000000000000000500000");
    await param2.type("0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c");
    await goto.check(page, "true");
  });
  /**
  * param1: hash rate is none
  * param2: clinet id is none
  */
  it("eth_submitHashrate_3", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "-32603");
  });

  /**
* param1: real hash rate
* param2: clinet id is none
*/
  it("eth_submitHashrate_4", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x0000000000000000000000000000000000000000000000000000000000500000");
    await testType.type("3"); // 0: none params 1: common params to request 2: more params
    await goto.check(page, "-32603");
  });
});
