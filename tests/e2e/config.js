import configSetting from "./config.json";

export default class Config {
  constructor() {
    this.axonRpc = configSetting.axonRpc;
    this.httpServer = configSetting.httpServer;
    this.hexPrivateKey = configSetting.hexPrivateKey;
    this.account1 = configSetting.account1;
    this.account2 = configSetting.account2;
  }

  static getIns() {
    if (!Config.ins) {
      Config.ins = new Config();
    }
    return Config.ins;
  }
}
