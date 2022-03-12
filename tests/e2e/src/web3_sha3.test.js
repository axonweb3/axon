import goto from "./goto";
import Config from "../config";

const pageName = "web3_sha3.html";
describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: Config.getIns().axonRpc.netWorkName,
      rpc: Config.getIns().axonRpc.url,
      chainId: Config.getIns().axonRpc.chainId,
    });
  });
  /**
  * param:0x68656c6c6f20776f726c64
  */
  test("web3_sha3_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("0x68656c6c6f20776f726c64");
    await goto.check(page, "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad");
  });

  /**
  * param:""
  */
  test("web3_sha3_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await param1.type("");
    await goto.check(page, "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
  });

  /**
 * none param
 */
  test("web3_sha3_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");// 0: none params  1：common params to request   2: more parameter
    await goto.check(page, "-32602");
  });

  /**
 *  param: 0x68656c6c6f20776f726c64
 *  more param
 */
  test("web3_sha3_4", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("0x68656c6c6f20776f726c64");
    await goto.check(page, "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad");
  });

  /**
 *  param (none hexadecimal string): 123456
 */
  test("web3_sha3_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("123456");
    await goto.check(page, "0xc888c9ce9e098d5864d3ded6ebcc140a12142263bace3a23a36f9905f12bd64a");
  });

  /**
 *  param(legal hexadecimal string without 0x prefix, even length):
 *  68656c6c6f20776f726c64
 */
  test("web3_sha3_6", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("68656c6c6f20776f726c64");
    await goto.check(page, "0xb1e9ddd229f9a21ef978f6fcd178e74e37a4fa3d87f453bc34e772ec91328181");
  });

  /**
 *  param(legal hexadecimal string with 0x prefix, odd length):
 *  Ox63646667781
 */
  test("web3_sha3_7", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("Ox63646667781");
    await goto.check(page, "0x9347910a2c4e4ecc48980bd0d2e01493d4d338ea84eeb596e029f679887ca4db");
  });
});
