import puppeteer from "puppeteer";
import { launch, setupMetamask, getMetamaskWindow } from "@chainsafe/dappeteer";

import Config from "../config";
import createTransactionData from "../src/create_test_data/createTestDataManage";

export const DAPPETEER_DEFAULT_CONFIG = {
  metamaskVersion: "v10.10.2",
  args: [
    process.env.HEADLESS ? "--headless=chrome" : "",
    process.env.WSL ? "-no-sandbox" : "",
  ],
};
export default async function setup() {
  const browser = await launch(puppeteer, DAPPETEER_DEFAULT_CONFIG);
  try {
    await createTransactionData.resetTestTmpFiles();
    await createTransactionData.createTransactionData(); // create test data
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
  
  await Config.getIns().initialize();
  await metamask.addNetwork({
    networkName: Config.getIns().axonRpc.netWorkName,
    rpc: Config.getIns().axonRpc.url,
    chainId: Config.getIns().axonRpc.chainId,
  });

  const page = await browser.newPage();
  await page.bringToFront();
  global.page = page;
}
