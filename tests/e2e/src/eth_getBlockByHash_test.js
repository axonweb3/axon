describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: "Axon",
      rpc: "http://localhost:8000",
      chainId: 5,
    });

    await Promise.all([
      page.goto("http://localhost:8080"),
      page.bringToFront(),
    ]);
  });

  const btnId = "#btn";
  const testType = document.getElementById("testType");
  const param1 = document.getElementById("param1");
  const ret = document.getElementById("ret");

  //
  test("eth_getBlockByHash_1", async () => {
    await page.click(btnId);
    testType.value.value = "1"; // 0: none params  1：common params to request   2: dvantage parameter
    param1.value = "";
    await page.waitForFunction(() => ret.innerText !== "");

    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe("");
  });

  //
  test("eth_getBlockByHash_2", async () => {
    await page.click(btnId);
    testType.value = "1";
    param1.value = "";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad",
    );
  });

  //
  test("eth_getBlockByHash_3", async () => {
    await page.click(btnId);
    testType.value = "1";
    param1.value = "";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });

  //    none param
  test("eth_getBlockByHash_4", async () => {
    await page.click(btnId);
    testType.value = "0";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });

  //    advantage  param
  test("eth_getBlockByHash_5", async () => {
    await page.click(btnId);
    testType.value = "2";
    param1.value = "";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });
});
