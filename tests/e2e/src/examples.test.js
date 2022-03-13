describe("Metamask", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080");
  });

  test("net_version", async () => {
    await page.click("#getChainId");

    await page.waitForFunction(
      () => document.getElementById("chainId").innerText !== "",
    );

    await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe("0x5");
  });
});
