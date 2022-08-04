import goto from "./goto";

const pageName = "eth_mining.html";

describe("eth_mining", () => {
  beforeAll(async () => {
    await goto.goto(page, pageName);
  });

  it("should returns false", async () => {
    await page.click("#getMining");

    await page.waitForFunction(
      () => document.getElementById("mining").innerText !== "",
    );

    await expect(page.$eval("#mining", (e) => e.innerText)).resolves.toBe("false");
  });
});
