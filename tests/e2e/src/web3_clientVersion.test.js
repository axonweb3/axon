describe("web3_clientVersion", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080/web3_clientVersion");
  });

  it("should returns MetaMask/v10.10.2", async () => {
    await page.click("#getClientVersion");

    await page.waitForFunction(
      () => document.getElementById("clientVersion").innerText !== "",
    );

    await expect(page.$eval("#clientVersion", (e) => e.innerText)).resolves.toBe("MetaMask/v10.10.2");
  });
});
