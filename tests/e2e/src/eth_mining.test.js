describe("eth_mining", () => {
  beforeAll(async () => {
    await page.goto("http://localhost:8080/eth_mining");
  });

  it("should returns false", async () => {
    await page.click("#getMining");

    await page.waitForFunction(
      () => document.getElementById("mining").innerText !== "",
    );

    await expect(page.$eval("#mining", (e) => e.innerText)).resolves.toBe("false");
  });
});
