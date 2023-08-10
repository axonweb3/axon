import goto from "./goto";

const pageName = "eth_feeHistory.html";

describe("eth_feeHistory", () => {
  beforeAll(async () => {
    // await page.goto("http://localhost:8080/eth_feeHistory");
    await goto.goto(page, pageName);
  });

  it("should returns normal fee history", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("1");
    await page.click("#btn");

    await page.waitForFunction(
      () => document.getElementById("ret").innerText !== "",
    );

    const result = page.$eval("#ret", (e) => e.innerText);

    await expect(result).resolves.not.toThrow();
    expect(JSON.parse(await result)).toMatchObject({
      baseFeePerGas: ["0x539", "0x539"],
      gasUsedRatio: [0],
      oldestBlock: "0x0",
      reward: [["0x24f304", "0x0"]],
    });
  }, 100000);

  it("should returns fee history without reward", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("2");
    await page.click("#btn");

    await page.waitForFunction(
      () => document.getElementById("ret").innerText !== "",
    );

    const result = page.$eval("#ret", (e) => e.innerText);

    await expect(result).resolves.not.toThrow();
    expect(JSON.parse(await result)).toMatchObject({
      baseFeePerGas: ["0x539", "0x539"],
      gasUsedRatio: [0],
      oldestBlock: "0x0",
    });
  }, 100000);

  it("should returns zero fee history", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("3");

    await page.waitForFunction(
      () => document.getElementById("ret").innerText !== "",
    );

    const result = page.$eval("#ret", (e) => e.innerText);

    await expect(result).resolves.not.toThrow();
    expect(JSON.parse(await result)).toMatchObject({
      gasUsedRatio: [0],
      oldestBlock: "0x0",
    });
  }, 100000);

  it("should returns second block fee history", async () => {
    const testType = await page.$(goto.pageIds.testTypeId);
    await testType.type("4");

    await page.waitForFunction(
      () => document.getElementById("ret").innerText !== "",
    );

    const result = page.$eval("#ret", (e) => e.innerText);

    await expect(result).resolves.not.toThrow();
    expect(JSON.parse(await result)).toMatchObject({
      baseFeePerGas: ["0x539", "0x539"],
      gasUsedRatio: [0],
      oldestBlock: "0x0",
      reward: [["0x24f304", "0x0"]],
    });
  }, 100000);
});
