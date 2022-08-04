import goto from "./goto";

const pageName = "eth_subscribe.html";

async function subscribe(buttonId, resultId) {
  await page.click(`#${buttonId}`);

  await page.waitForFunction(
    (id) => document.getElementById(id).innerText !== "",
    {},
    resultId,
  );

  const request = page.$eval(`#${resultId}`, (e) => e.innerText);
  await expect(request).resolves.not.toThrow();

  const result = await request;
  expect(result).toHaveLength(34);
  expect(result.substr(0, 2)).toBe("0x");

  return result;
}

async function unsubscribe(id) {
  await page.type("#unsubscribeId", id);
  await page.click("#unsubscribe");

  await page.waitForFunction(
    () => document.getElementById("unsubscribeResult").innerText !== "",
  );

  await expect(page.$eval("#unsubscribeResult", (e) => e.innerText)).resolves.toBe("true");
}

describe("eth_subscribe", () => {
  beforeAll(async () => {
    await goto.goto(page, pageName);
  });

  it("should successfully subscribe new heads", async () => {
    const result = await subscribe("subscribeNewHeads", "newHeads");
    await unsubscribe(result);
  });

  it.skip("should successfully subscribe syncing", async () => {
    const result = await subscribe("subscribeSyncing", "syncing");
    await unsubscribe(result);
  });

  it.skip("should successfully subscribe logs", async () => {
    const result = await subscribe("subscribeLogs", "logs");
    await unsubscribe(result);
  });
});
