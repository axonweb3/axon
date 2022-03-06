import puppeteer from "puppeteer";

import { launch, setupMetamask } from "@chainsafe/dappeteer";

export const DAPPETEER_DEFAULT_CONFIG = { metamaskVersion: "v10.8.1" };

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
}
