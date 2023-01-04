import puppeteer from "puppeteer";
import { bootstrap,launch, setupMetamask, getMetamaskWindow } from "@chainsafe/dappeteer";
import {RECOMMENDED_METAMASK_VERSION} from "@chainsafe/dappeteer/dist/index";

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
export const DappeteerLaunchOptions = {
  automation: "puppeteer",
    browser: "chrome",
    metaMaskVersion: RECOMMENDED_METAMASK_VERSION,
    puppeteerOptions: {
      args: [
        process.env.HEADLESS ? "--headless=chrome" : "",
        process.env.WSL ? "-no-sandbox" : "",
      ],
    },
};
export const MetaMaskOptions = {
  seed: "bubble young armed shed unusual acid pilot chase caught crop defense only",
  password: "12345678",
  showTestNets: true,
};
export default async function setup() {
  // const browser = await launch(puppeteer, DAPPETEER_DEFAULT_CONFIG);
  // const browser = await launch(puppeteer, DAPPETEER_DEFAULT_CONFIG);
  // const { metaMask, browser } = await dappeteer.bootstrap();
  // const { metaMask,browser, metaMaskPage } = await bootstrap(DappeteerLaunchOptions&MetaMaskOptions);
  // const { metaMask,browser } = await bootstrap(DappeteerLaunchOptions&MetaMaskOptions);
  // const { metaMask,browser, metaMaskPage} = await bootstrap(puppeteer, { seed: "bubble young armed shed unusual acid pilot chase caught crop defense only", password: 12345678, metamaskVersion: RECOMMENDED_METAMASK_VERSION });
  // const { metaMask,browser } = await bootstrap();
  const [metaMask, metaMaskPage ,browser] = await bootstrap(puppeteer, {
    seed: "bubble young armed shed unusual acid pilot chase caught crop defense only", 
    password: "12345678", 
    metamaskVersion: RECOMMENDED_METAMASK_VERSION });
  console.log('begin setup metamask...');
  // const browser = await dappeteer.launch(puppeteer, {
  //   metamaskVersion: 'v10.15.0',
  //   // defaultViewport: { width: 1920, height: 1080 },
  //   headless: false, // limitation of puppeteer, chrome can't be used in headless mode
  // });

  // console.log('Loading metamask...');
  // const metamask = await setupMetamask(browser, {
  //     seed: "bubble young armed shed unusual acid pilot chase caught crop defense only",
  //     password: "12345678",
  //     showTestNets: false,
  //     hideSeed: true,
  // });
  console.log('go on Loading metamask...');
  try {
    await createTransactionData.resetTestTmpFiles();
    await createTransactionData.createTransactionData(); // create test data
    // await setupMetamask(browser, {}); //
    global.browser = browser;
  } catch (error) {
    // eslint-disable-next-line no-console
    console.log(error);
    throw error;
  }
  
  process.env.PUPPETEER_WS_ENDPOINT = browser.wsEndpoint();
  console.log('is there...?');
  global.browser = browser;
  // global.metamask = await getMetamaskWindow(browser);
  global.metamask = metaMask;

  // await Config.getIns().initialize();
  console.log('is there...?');
  const x = Config.getIns().axonRpc;
  console.log(x);
  // await metaMask.addNetwork({
  await metamask.addNetwork({
    networkName: Config.getIns().axonRpc.netWorkName,
    rpc: Config.getIns().axonRpc.url,
    chainId: Config.getIns().axonRpc.chainId,
    // chainId: 5,
  });
  console.log('is there...?');
  const page = await browser.newPage();
  await page.bringToFront();
  global.page = page;
  console.log('is there...?');
}
