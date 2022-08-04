import configSetting from "./config.json";

function makeRequest(method, url) {
  return new Promise((resolve, reject) => {
    const XMLHttpRequest = require("xhr2");// eslint-disable-line global-require
    const xhr = new XMLHttpRequest();
    xhr.open(method, url);
    xhr.setRequestHeader("Content-type", "application/json");
    xhr.send(JSON.stringify(
      {
        id: 1,
        jsonrpc: "2.0",
        method: "eth_chainId",
        params: [

        ],
      },
    ));
    xhr.onload = function load_() {
      if (this.status >= 200 && this.status < 300) {
        resolve(xhr.response);
      } else {
        reject(new Error("makeRequest fail"));
      }
    };
    xhr.onerror = function error_() {
      // reject({
      //   status: this.status,
      //   statusText: xhr.statusText,
      // });
      reject(new Error("makeRequest fail"));
    };
    // xhr.send();
  });
}

async function doAjaxThings() {
  const result = await makeRequest("POST", configSetting.axonRpc.url);

  return new Promise((resolve) => {
    resolve(parseInt(JSON.parse(result).result, 16));
  });
}

export default class Config {
  constructor() {
    this.axonRpc = { url: "", netWorkName: "", chainId: "" };
    this.acount1 = "";
    this.acount2 = "";
    this.httpServer = "";
    this.hexPrivateKey = "";
    try {
      // eslint-disable-next-line no-console
      console.log(configSetting.axonRpc);
      this.axonRpc = configSetting.axonRpc;
      this.httpServer = configSetting.httpServer;
      this.hexPrivateKey = configSetting.hexPrivateKey;
      this.acount1 = configSetting.acount1;
      this.acount2 = configSetting.acount2;
    } catch (err) {
      // eslint-disable-next-line no-console
      console.log(err);
      throw err;
    }
  }

  async initialize() {
    this.axonRpc.chainId = await doAjaxThings();
  }

  static getIns() {
    if (!Config.ins) {
      Config.ins = new Config();
    }
    (async function init_() {
      await Config.ins.initialize();
    }());
    return Config.ins;
  }
}
