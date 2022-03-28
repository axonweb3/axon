import fs from "fs";

export default class Config {
  constructor() {
    this.axonRpc = { url: "", netWorkName: "", chainId: "" };
    this.acount1 = "";
    this.acount2 = "";
    this.httpServer = "";
    this.hexPrivateKey = "";
    try {
      const buffer = fs.readFileSync("config.json", "utf8");
      const configSetting = JSON.parse(buffer);
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

  static getIns() {
    if (!Config.ins) {
      Config.ins = new Config();
    }
    return Config.ins;
  }
}
