import { bootstrap } from "dappeteer-new";
import { RECOMMENDED_METAMASK_VERSION } from "dappeteer-new/dist/index";

import Config from "../config";
import createTransactionData from "../src/create_test_data/createTestDataManage";

export const MetaMaskOptions = {
  metaMaskVersion: RECOMMENDED_METAMASK_VERSION,
  automation: "puppeteer",
  // https://developer.chrome.com/articles/new-headless/
  headless: process.env.HEADLESS ? 'new' : false,
  metaMaskFlask: false,
  args: [
    process.env.WSL ? "-no-sandbox" : "",
  ]
};

export default async function setup() {
  console.log('Setup Start...');

  const { metaMask, browser } = await bootstrap(MetaMaskOptions);
  let hostPage;
  try {
    await createTransactionData.resetTestTmpFiles();
    await createTransactionData.createTransactionData(); // create test data

    process.env.PUPPETEER_WS_ENDPOINT = browser.wsEndpoint();
    global.browser = browser;
    global.metamask = metaMask;

    console.log(`browser.newPage()`);
    hostPage = await browser.newPage().catch(err => {
      console.error("browser.newPage() failed");
      throw err;
    });
  } catch (error) {
    // eslint-disable-next-line no-console
    console.log(error);
    throw error;
  }

  console.log(`goto httpServer: ${Config.getIns().httpServer}`);
  await hostPage.goto(Config.getIns().httpServer);

  const configParams = {
    networkName: Config.getIns().axonRpc.netWorkName,
    rpc: process.env.AXON_RPC_URL || Config.getIns().axonRpc.url,
    chainId: "0x" + Config.getIns().axonRpc.chainId.toString(16),
    symbol: "AXON"
  }
  console.log("MetaMask configs:", configParams);

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

  console.log('Setup End.');
}
