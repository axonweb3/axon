describe("net_version", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080/net_version");
  });

  it("should returns 0x05", async () => {
    await page.click("#getChainId");

    await page.waitForFunction(
      () => document.getElementById("chainId").innerText !== "",
    );

    await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe(
      "0x5",
    );
  });
});
