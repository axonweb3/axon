import Config from "../config";

const goto = {
  pageIds: {
    btnId: "#btn", testTypeId: "#testType", param1Id: "#param1", param2Id: "#param2", param3Id: "#param3", param4Id: "#param4", param5Id: "#param5", param6Id: "#param6", param7Id: "#param7", param8Id: "#param8",
  },
  async goto(currentpage, pageName) {
    try {
      // await currentpage.goto(`${Config.getIns().httpServer}/src/${pageName}`);
      await currentpage.goto(`${Config.getIns().httpServer}/${pageName}`);
    } catch (err) {
      // eslint-disable-next-line no-console
      console.log(err);
      throw err;
    }
  },

  async check(currentpage, expectedValue) {
    await currentpage.click(goto.pageIds.btnId);
    await currentpage.waitForFunction(() => document.getElementById("ret").innerText !== "");
    await expect(currentpage.$eval("#ret", (e) => e.innerText)).resolves.toMatch(expectedValue);
  },
  // get the  value
  async value(currentpage) {
    await currentpage.click(goto.pageIds.btnId);
    await currentpage.waitForFunction(() => document.getElementById("ret").innerText !== "");
    return currentpage.$eval("#ret", (e) => e.innerText);
  },
};
export default goto;
