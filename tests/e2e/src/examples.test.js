describe("Metamask", () => {
  beforeAll(async () => {
    await metamask.addNetwork({
      networkName: "Axon",
      rpc: "http://localhost:8000",
      chainId: 5,
    });

    await Promise.all([page.goto("http://localhost:8080"), page.bringToFront()]);
  });

  test("net_version", async () => {
    await page.click("#getChainId");

    await page.waitForFunction(
      () => document.getElementById("chainId").innerText !== "",
    );

    await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe("0x5");
  });
});
