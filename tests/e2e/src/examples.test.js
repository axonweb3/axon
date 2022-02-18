import { getRequester, testAssertMatchObject } from "./utils.js";

testAssertMatchObject(
  "GET / fully match",
  getRequester().get("/"),
  { data: "Hi\n" },
);

test(
  "GET / conditionally match",
  async () => {
    expect((await getRequester().get("/")).data).toContain("Hi");
  },
);
