import puppeteer from "puppeteer";
import { bootstrap,launch, setupMetamask, getMetamaskWindow } from "@chainsafe/dappeteer";

import Config from "../config";
import createTransactionData from "../src/create_test_data/createTestDataManage";

// export const DAPPETEER_DEFAULT_CONFIG = {
//   metamaskVersion: "v10.23.2",
//   args: [
//     process.env.HEADLESS ? "--headless=chrome" : "",
//     process.env.WSL ? "-no-sandbox" : "",
//   ],
// };
export const DAPPETEER_DEFAULT_CONFIG = {
  metamaskVersion: "v10.23.0",
  args: [
    process.env.HEADLESS ? "--headless=chrome" : "",
    process.env.WSL ? "-no-sandbox" : "",
  ],
};
export default async function setup() {
  // const browser = await launch(puppeteer, DAPPETEER_DEFAULT_CONFIG);
  // const browser = await launch(puppeteer, DAPPETEER_DEFAULT_CONFIG);
  // const { metaMask, browser } = await dappeteer.bootstrap();
  const { browser, metaMask, metaMaskPage } = await bootstrap({
    automation: "puppeteer",
    browser: "chrome",
    metaMaskVersion: "v10.14.0",
    puppeteerOptions: {
      args: [
        process.env.HEADLESS ? "--headless=chrome" : "",
        process.env.WSL ? "-no-sandbox" : "",
      ],
    },
  });
  // const { metaMask,browser } = await bootstrap();
  try {
    await createTransactionData.resetTestTmpFiles();
    await createTransactionData.createTransactionData(); // create test data
    // await setupMetamask(browser, {});
    global.browser = browser;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.log(error);
    throw error;
  }
  process.env.PUPPETEER_WS_ENDPOINT = browser.wsEndpoint();

  global.browser = browser;
  // global.metamask = await getMetamaskWindow(browser);
  global.metamask = metaMask;

  // await Config.getIns().initialize();
  await metamask.addNetwork({
    networkName: Config.getIns().axonRpc.netWorkName,
    rpc: Config.getIns().axonRpc.url,
    chainId: Config.getIns().axonRpc.chainId,
    // chainId: 5,
  });

  const page = await browser.newPage();
  await page.bringToFront();
  global.page = page;
}
