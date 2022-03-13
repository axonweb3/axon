describe("eth_syncing", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080/eth_syncing");
  });

  it("should returns false", async () => {
    await page.click("#getSyncing");

    await page.waitForFunction(
      () => document.getElementById("syncing").innerText !== "",
    );

    await expect(page.$eval("#syncing", (e) => e.innerText)).resolves.toBe("false");
  });
});
