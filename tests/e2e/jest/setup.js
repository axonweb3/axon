import puppeteer from "puppeteer";
import { bootstrap } from "@chainsafe/dappeteer";
import { RECOMMENDED_METAMASK_VERSION } from "@chainsafe/dappeteer/dist/index";

import Config from "../config";
import createTransactionData from "../src/create_test_data/createTestDataManage";

export const MetaMaskOptions = {
  // empty wallet
  seed: "bubble young armed shed unusual acid pilot chase caught crop defense only",
  password: "12345678",
  metamaskVersion: RECOMMENDED_METAMASK_VERSION,
  args: [
    process.env.HEADLESS ? "--headless=new" : "",
    process.env.WSL ? "-no-sandbox" : "",
  ]
};
export default async function setup() {
  const [metaMask, ,browser] = await bootstrap(puppeteer, MetaMaskOptions);
  try {
    await createTransactionData.resetTestTmpFiles();
    await createTransactionData.createTransactionData(); // create test data
    global.browser = browser;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.log(error);
    throw error;
  }
  
  process.env.PUPPETEER_WS_ENDPOINT = browser.wsEndpoint();
  global.browser = browser;
  global.metamask = metaMask;

  // await Config.getIns().initialize();
  await metamask.addNetwork({
    networkName: Config.getIns().axonRpc.netWorkName,
    rpc: Config.getIns().axonRpc.url,
    chainId: Config.getIns().axonRpc.chainId,
    symbol: "AXON"
  });
  const page = await browser.newPage();
  await page.bringToFront();
  global.page = page;
}
