import goto from "./goto";

const pageName = "eth_syncing.html";

describe("eth_syncing", () => {
  beforeAll(async () => {
    await goto.goto(page, pageName);
  });

  it("should returns false", async () => {
    await page.click("#getSyncing");

    await page.waitForFunction(
      () => document.getElementById("syncing").innerText !== "",
    );

    await expect(page.$eval("#syncing", (e) => e.innerText)).resolves.toBe("false");
  });
});
