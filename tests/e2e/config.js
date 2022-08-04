import configSetting from "./config.json";

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
    if (!Config.ins) {
      Config.ins = new Config();
    }
    Config.ins.axonRpc.chainId = await doAjaxThings();
    console.log(Config.ins.axonRpc.chainId);
    console.log("Config.ins.axonRpc.chainId");
  }

  // static getIns() {
  //   if (!Config.ins) {
  //     Config.ins = new Config();
  //   }
  //   (async function () {
  //     await Config.ins.initialize();
  //   })();
  //   return Config.ins;
  // }

  static getIns() {
    if (!Config.ins) {
      Config.ins = new Config();
    }
    return Config.ins;
  }
}

async function doAjaxThings() {
  // await code here
  const result = await makeRequest("POST", configSetting.axonRpc.url);
  // code below here will only execute when await makeRequest() finished loading
  console.log(result);
  // return parseInt(JSON.parse(result).result, 16);

  return new Promise((resolve, reject) => {
    resolve(parseInt(JSON.parse(result).result, 16));
  });
}

function makeRequest(method, url) {
  return new Promise((resolve, reject) => {
    const XMLHttpRequest = require("xhr2");
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
    xhr.onload = function () {
      if (this.status >= 200 && this.status < 300) {
        resolve(xhr.response);
      } else {
        reject({
          status: this.status,
          statusText: xhr.statusText,
        });
      }
    };
    xhr.onerror = function () {
      reject({
        status: this.status,
        statusText: xhr.statusText,
      });
    };
    // xhr.send();
  });
}
