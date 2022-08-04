import goto from "./goto";

const pageName = "eth_feeHistory.html";

describe("eth_feeHistory", () => {
  beforeAll(async () => {
    // await page.goto("http://localhost:8080/eth_feeHistory");
    await goto.goto(page, pageName);
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
  }, 100000);
});
