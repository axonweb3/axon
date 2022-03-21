describe("net_peerCount", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080/net_peerCount");
  });

  it("should returns 0x0", async () => {
    await page.click("#getPeerCount");

    await page.waitForFunction(
      () => document.getElementById("peerCount").innerText !== "",
    );

    await expect(page.$eval("#peerCount", (e) => e.innerText)).resolves.toBe("0x0");
  });
});
