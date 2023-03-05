import { bootstrap } from "@chainsafe/dappeteer";
import { RECOMMENDED_METAMASK_VERSION } from "@chainsafe/dappeteer/dist/index";

import Config from "../config";
import createTransactionData from "../src/create_test_data/createTestDataManage";

export const MetaMaskOptions = {
  metaMaskVersion: RECOMMENDED_METAMASK_VERSION,
  automation: "puppeteer",
  headless: process.env.HEADLESS ? true : false,
  metaMaskFlask: false,
  args: [
    process.env.WSL ? "-no-sandbox" : "",
  ]
};
export default async function setup() {
  const { metaMask, browser } = await bootstrap(MetaMaskOptions);
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

  const hostPage = await browser.newPage();
  await hostPage.goto("http://localhost:8080");
  await Config.getIns().initialize();
  const configParams = {
    networkName: Config.getIns().axonRpc.netWorkName,
    rpc: Config.getIns().axonRpc.url,
    chainId: "0x" + Config.getIns().axonRpc.chainId.toString(16),
    symbol: "AXON"
  }
  // add custom network to a MetaMask
  await hostPage.evaluate((cfparams) => {
    window.ethereum.request({
      method: "wallet_addEthereumChain",
      params: [
        {
          chainId: cfparams.chainId,
          chainName: cfparams.networkName,
          nativeCurrency: {
            name: "Axon",
            symbol: "Axon", // 2-6 characters long
            decimals: 18,
          },
          rpcUrls: [cfparams.rpc],
        },
      ],
    });
  }, configParams);
  await metaMask.acceptAddNetwork(false);
  await metaMask.switchNetwork("Axon");
  
  await hostPage.bringToFront();
  global.page = hostPage.page;
}
