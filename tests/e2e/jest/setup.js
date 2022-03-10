import puppeteer from "puppeteer";

import { launch, setupMetamask, getMetamaskWindow } from "@chainsafe/dappeteer";

export const DAPPETEER_DEFAULT_CONFIG = { metamaskVersion: "v10.8.1", args: ["--headless=chrome", "--no-sandbox"] };

export default async function setup() {
  const browser = await launch(puppeteer, DAPPETEER_DEFAULT_CONFIG);
  try {
    await setupMetamask(browser, {});
    global.browser = browser;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.log(error);
    throw error;
  }
  process.env.PUPPETEER_WS_ENDPOINT = browser.wsEndpoint();

  global.browser = browser;
  global.metamask = await getMetamaskWindow(browser);

  await metamask.addNetwork({
    networkName: "Axon",
    rpc: "http://localhost:8000",
    chainId: 5,
  });

  const page = await browser.newPage();
  await page.bringToFront();
  global.page = page;
}
