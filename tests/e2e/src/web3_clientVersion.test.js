import goto from "./goto";

const pageName = "web3_clientVersion.html";

describe("web3_clientVersion", () => {
  beforeAll(async () => {
    await goto.goto(page, pageName);
  });

  it("should returns MetaMask/v10.10.2", async () => {
    await page.click("#getClientVersion");

    await page.waitForFunction(
      () => document.getElementById("clientVersion").innerText !== "",
    );

    await expect(page.$eval("#clientVersion", (e) => e.innerText)).resolves.toBe("MetaMask/v10.10.2");
  });
});
