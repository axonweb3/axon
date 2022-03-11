const host = "http://localhost:8080";
const axonRpc = { url: "http://localhost:8000", netWrokName: "axon", chianId: 5 };
const pageIds = {
  btnId: "#btn", testTypeId: "#testType", param1Id: "#param1", param2Id: "#param2", param3Id: "#param3", param4Id: "#param4",
};
const goto = async (currentpage, pageName) => {
  try {
    await Promise.all([
      currentpage.goto(`${host}/${pageName}`),
      currentpage.bringToFront(),
    ]);
  } catch (ex) {
    console.log(ex);
  }
};

const check = async (currentpage, expectedValue) => {
  await currentpage.click(pageIds.btnId);
  await currentpage.waitForFunction(() => document.getElementById("ret").innerText !== "");
  await expect(currentpage.$eval("#ret", (e) => e.innerText)).resolves.toMatch(expectedValue);
};
module.exports.goto = {
  goto, check, pageIds, host, axonRpc,
};
