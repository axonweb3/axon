describe("eth_feeHistory", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080/eth_feeHistory");
  });

  it("should returns empty fee history", async () => {
    await page.click("#getFeeHistory");

    await page.waitForFunction(
      () => document.getElementById("feeHistory").innerText !== "",
    );

    const result = page.$eval("#feeHistory", (e) => e.innerText);

    await expect(result).resolves.not.toThrow();
    expect(JSON.parse(await result)).toMatchObject({
      baseFeePerGas: [],
      gasUsedRatio: [],
      oldestBlock: "0x0",
      reward: null,
    });
  });
});
