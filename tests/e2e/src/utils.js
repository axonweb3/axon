import Axios from "axios";
import Util from "util";

const TESTS = [];

const REQUESTER = Axios.create({ baseURL: "http://localhost:8080" });

export function getRequester() {
  return REQUESTER;
}

export async function testAssertMatchObject(msg, a, b) {
  test(msg, () => expect(a).resolves.toMatchObject(b));
}
