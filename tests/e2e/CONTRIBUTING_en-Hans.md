# Test Case Style Guide

## Assert function

Prefer to use the assert function provided by `Jest` instead of handling the judgment yourself.

#### Definition

`Jest` provides a series of `expect` assertion functions to check whether the results of the code meet expectations.
* <https://jestjs.io/docs/expect>

#### Advantage

- The assertion function provides detailed error information to help better locate errors.
- `Jest` provides a series of preset inspection methods to facilitate writing common judgment scenarios.
- The naming of assertion functions is semantic, and even people unfamiliar with the code can read the rules for their construction.

#### Disadvantage

- Assertion functions are not native components of `JavaScript`, there is a cost to learn them.
- Assertion functions cannot handle all complex cases.

#### Conclusion

Assertion functions should be used as much as possible. If there are special cases, use [`expect.extend`](https://jestjs.io/docs/expect#expectextendmatchers) to add new assertion functions.

#### Example

```javascript
// Good
test("1 + 1 = 2", () => {
  expect(1 + 1).toBe(2);
});

// Bad: the condition is not handled by the predicate function
test("1 + 1 = 2", () => {
  expect(1 + 1 === 2).toBe(true);
});

// Good
test("version should be 5", async () => {
  await expect(getVersion()).resolves.toBe(5);
});

// Bad: using `.resolves` can provide more error messages
test("version should be 5", async () => {
  expect(await getVersion()).toBe(5);
});
```

## Description by group

For test cases in different situations of the same scenario, they tend to be placed under the same `describe` description.

#### Definition

The `describe` function can put multiple test cases in the same group.
* <https://jestjs.io/docs/api#describename-fn>

#### Advantage

- The `describe` function can provide a clearer list of test output.
- The same group of test cases can control the behavior uniformly, using `afterAll`, `afterEach`, `beforeAll`, or `beforeEach` functions.
- Test cases using the same test method but with different data can be grouped together by `describe.each`.

## Test case description

Each test case description should include test objects, inputs and outputs that do not appear in the group description.

#### Example

```javascript
// Good
test("getVersion() should returns 5", () => {
  expect(getVersion()).toBe(5);
});

// Good
describe("getVersion()", () => {
  it("should returns 5", () => {
    expect(getVersion()).toBe(5);
  });
});

// Bad: Does not describe the output of the test
test("getVersion()", () => {
  expect(getVersion()).toBe(5);
});
```

## Function alias

Use function aliases based on semantic scenarios, tend to use short function aliases. Do not use non-semantic `Jest` function aliases such as `xdescribe`.

#### Definition

`Jest` provides aliases for a number of functions to improve readability.

#### Advantage

- Using different function aliases in different scenarios can make the code closer to natural language.
- Function aliases can shorten the length of the code and make it easier to read.

#### Disadvantage

- Non-semantic function aliases are not conducive to people who are not familiar with the `Jest` framework to understand the code.

#### Conclusion

Improve code readability by using function aliases. If code readability cannot be improved, function aliases should be avoided.

#### Example

```javascript
// Good
test("drinks returns", () => {
  const drink = jest.fn(() => true);
  drink();
  expect(drink).toReturn();
});

// Not so good: extra function name length, but provides no meaningful information
test("drinks returns", () => {
  const drink = jest.fn(() => true);
  drink();
  expect(drink).toHaveReturned();

});

// Good
describe.only("my beverage", () => {
  it("is delicious", () => {
    expect(myBeverage.delicious).toBeTruthy();
  });
});

// Not so good: it is delicious is closer to natural language writing
describe.only("my beverage", () => {
  test("is delicious", () => {
    expect(myBeverage.delicious).toBeTruthy();
  });
});

// Bad: incomprehensible name
fdescribe("my beverage", () => {
  it("is delicious", () => {
    expect(myBeverage.delicious).toBeTruthy();
  });
});
```

## Wait page

Use the wait method provided by `Puppeteer`.

#### Definition

`Puppeteer` comes with a series of wait methods to wait for the page to reach the state we need.

* <https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForSelector>
* <https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForXPath>
* <https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForFunction>
* <https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForTimeout>

#### Advantage

- These wait functions provide a series of basic functions such as query interval, timeout period and element attributes.
- Blocking on the test will end shortly after the condition is met.
- It is convenient to uniformly adjust the waiting parameters.

#### Disadvantage

- Inside these wait functions, some objects cannot be used, such as `page`.
- More complex than a simple delay.

#### Conclusion

For the stability and performance of the test, I tend to use the wait method provided by `Puppeteer`.

#### Example

```javascript
// good
test("net_version should returns 0x5", async () => {
  await page.click("#getChainId");

  await page.waitForFunction(
    () => document.getElementById("chainId").innerText !== "",
  );

  await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe("0x5");
});

// Bad: rigid waiting
test("net_version should returns 0x5", async () => {
  await page.click("#getChainId");

  await sleep(1000);

  await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe("0x5");
});
```
