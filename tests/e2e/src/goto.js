// eslint-disable-next-line import/no-import-module-exports
import { Config } from "../config";

const pageIds = {
  btnId: "#btn", testTypeId: "#testType", param1Id: "#param1", param2Id: "#param2", param3Id: "#param3", param4Id: "#param4",
};
const goto = async (currentpage, pageName) => {
  try {
    await Promise.all([
      currentpage.goto(`${Config.getIns().httpServer}/${pageName}`),
      currentpage.bringToFront(),
    ]);
  } catch (ex) {
    // eslint-disable-next-line no-console
    console.log(ex);
  }
};

const check = async (currentpage, expectedValue) => {
  await currentpage.click(pageIds.btnId);
  await currentpage.waitForFunction(() => document.getElementById("ret").innerText !== "");
  await expect(currentpage.$eval("#ret", (e) => e.innerText)).resolves.toMatch(expectedValue);
};
module.exports.goto = {
  goto, check, pageIds,
};
