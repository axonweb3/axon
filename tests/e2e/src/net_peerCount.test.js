import goto from "./goto";

const pageName = "net_peerCount.html";

describe("net_peerCount", () => {
  beforeAll(async () => {
    await goto.goto(page, pageName);
  });

  it("should returns 0x0", async () => {
    await page.click("#getPeerCount");

    await page.waitForFunction(
      () => document.getElementById("peerCount").innerText !== "",
    );

    await expect(page.$eval("#peerCount", (e) => e.innerText)).resolves.toBe("0x0");
  });
});
