// eslint-disable-next-line
import { goto } from "./goto";
const pageName = "web3_sha3.html";
describe("web3_sha3", () => {
  /**
  * param: 0x68656c6c6f20776f726c64
  */
  test("web3_sha3_1", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("0x68656c6c6f20776f726c64");
    await goto.check(page, "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad");
  });

  /**
  * param: ""
  */
  test("web3_sha3_2", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    const param1 = await page.$(goto.pageIds.param1Id);
    await testType.type("1"); // 0: none params 1: common params to request 2: more params
    await param1.type("");
    await goto.check(page, "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
  });

  /**
  * none param
  */
  test("web3_sha3_3", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("0"); // 0: none params 1: common params to request 2: more params
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
  *  param: none hexadecimal string 123456
  */
  test("web3_sha3_5", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("123456");
    await goto.check(page, "0x6adf031833174bbe4c85eafe59ddb54e6584648c2c962c6f94791ab49caa0ad4");
  });

  /**
  *  param: legal hexadecimal string without 0x prefix, even length 68656c6c6f20776f726c64
  */
  test("web3_sha3_6", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("68656c6c6f20776f726c64");
    await goto.check(page, "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad");
  });

  /**
  *  param: legal hexadecimal string with 0x prefix, odd length Ox63646667781
  */
  test("web3_sha3_7", async () => {
    await goto.goto(page, pageName);
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");
    const param1 = await page.$(goto.pageIds.param1Id);
    await param1.type("Ox63646667781");
    await goto.check(page, "-32602");
  });
});
