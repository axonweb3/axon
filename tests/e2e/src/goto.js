import Config from "../config";

export const pageIds = {
  btnId: "#btn", testTypeId: "#testType", param1Id: "#param1", param2Id: "#param2", param3Id: "#param3", param4Id: "#param4",
};

export async function goto(currentpage, pageName) {
  try {
    await currentpage.goto(`${Config.getIns().httpServer}/${pageName}`);
  } catch (ex) {
    // eslint-disable-next-line no-console
    console.log(ex);
  }
}

export async function check(currentpage, expectedValue) {
  await currentpage.click(pageIds.btnId);
  await currentpage.waitForFunction(() => document.getElementById("ret").innerText !== "");
  await expect(currentpage.$eval("#ret", (e) => e.innerText)).resolves.toMatch(expectedValue);
}
