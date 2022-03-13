import NodeEnvironment from "jest-environment-node";

export default class DappeteerEnvironment extends NodeEnvironment {
  async setup() {
    await super.setup();

    this.global.browser = global.browser;
    this.global.metamask = global.metamask;
    this.global.page = global.page;
  }
}
