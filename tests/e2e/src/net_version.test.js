import goto from "./goto";

const pageName = "net_version.html";

describe("net_version", () => {
  beforeAll(async () => {
    await goto.goto(page, pageName);
  });

  it("should returns version", async () => {
    await page.click("#getChainId");

    await page.waitForFunction(
      () => document.getElementById("chainId").innerText !== "",
    );

    await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe(2022);
  });
});
