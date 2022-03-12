import goto from "./goto";
import Config from "../config";

const pageName = "eth_getTransactionByBlockHashAndIndex.html";
describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: Config.getIns().axonRpc.netWorkName,
      rpc: Config.getIns().axonRpc.url,
      chainId: Config.getIns().axonRpc.chainId,
    });
  });
  /**
  * param1:0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876
  * param2:0x1
  */
  test("eth_getTransactionByBlockHashAndIndex_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    const param2 = await page.$(goto.pageIds.param2Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x24e5f68936e20c7d2aef3438937373642bc5ea582e16458f4b1fdad855252876");
    await param2.type("0x1");
    await goto.check(page, "0x1");
  });
});
