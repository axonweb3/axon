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

  //    the  parameter is empty
  test("web3_sha3_1", async () => {
    await page.click(btnId);
    testType.value.value = "1"; // 0: none params  1：common params to request   2: dvantage parameter
    param1.value = "";
    await page.waitForFunction(() => ret.innerText !== "");

    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
    );
  });

  //    the  parameter is 0x68656c6c6f20776f726c64
  test("web3_sha3_2", async () => {
    await page.click(btnId);
    testType.value = "1";
    param1.value = "0x68656c6c6f20776f726c64";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad",
    );
  });

  //    is not hex  param
  test("web3_sha3_3", async () => {
    await page.click(btnId);
    testType.value = "1";
    param1.value = "123456789";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });

  //    legal hexadecimal string without 0x prefix, even length
  test("web3_sha3_4", async () => {
    await page.click(btnId);
    testType.value = "1";
    param1.value = "68656c6c6f20776f726c64"; // fix me
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });

  //    legal hexadecimal string without 0x prefix, length is odd
  test("web3_sha3_5", async () => {
    await page.click(btnId);
    testType.value = "1";
    param1.value = "68656c6c6f20776f726c64c"; // fix me
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });

  //    none param
  test("web3_sha3_6", async () => {
    await page.click(btnId);
    testType.value = "0"; // fix me
    param1.value = "";
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });

  //    advantage  param
  test("web3_sha3_7", async () => {
    await page.click(btnId);
    testType.value = "2";
    param1.value = ""; //   fix me
    await page.waitForFunction(() => ret.innerText !== "");
    await expect(page.$eval(ret.id, (e) => e.innerText)).resolves.toBe(
      "", // fix me
    );
  });
});
