## 断言函数

倾向于使用 `Jest` 提供的断言函数，而非自行处理判断。

#### 定义

`Jest` 提供了一系列的 `expect` 断言函数，用来检查代码的结果是否符合预期。
* https://jestjs.io/docs/expect

#### 优势

- 断言函数提供了详细的错误信息，帮助更好定位错误。
- `Jest` 提供了一系列预置的检查方法，方便书写常见的判断场景。
- 断言函数的命名是语义化的，即使不熟悉代码的人也能阅读它们构造的规则。

#### 劣势

- 断言函数并非 `JavaScript` 原生组件，学习它们需要付出成本。
- 断言函数并不能处理所有的复杂情况。

#### 结论

断言函数应当被尽可能地使用，如果有特殊情况，使用 [`expect.extend`](https://jestjs.io/docs/expect#expectextendmatchers) 来添加新的断言函数。

#### 例子

```javascript
// 好
test("1 + 1 = 2", () => {
  expect(1 + 1).toBe(2);
});

// 坏：判断条件并非由断言函数处理
test("1 + 1 = 2", () => {
  expect(1 + 1 === 2).toBe(true);
});

// 好
test("version should be 5", async () => {
  await expect(getVersion()).resolves.toBe(5);
});

// 坏：使用 `.resolves` 能提供更多的错误信息
test("version should be 5", async () => {
  expect(await getVersion()).toBe(5);
});
```

## 分组描述

对于同一个场景不同情况的测试用例，倾向于把它们放在同一个 `describe` 描述下。

#### 定义

`describe` 函数可以将多个测试用例放在同一组内。
* https://jestjs.io/docs/api#describename-fn

#### 优势

- `describe` 函数可以提供更清晰的测试输出列表。
- 同一组测试用例可以统一控制行为，使用 `afterAll`, `afterEach`, `beforeAll`, 或 `beforeEach` 函数。
- 使用同样测试方式，但不同数据的测试用例可以通过 `describe.each` 组织在一起。

## 用例描述

在每个用例描述中应包含测试的对象，输入以及输出，除非它们已经在分组描述中出现过。

#### 例子

```javascript
// 好
test("getVersion() should returns 5", () => {
  expect(getVersion()).toBe(5);
});

// 好
describe("getVersion()", () => {
  it("should returns 5", () => {
    expect(getVersion()).toBe(5);
  });
});

// 坏：没有描述测试的输出
test("getVersion()", () => {
  expect(getVersion()).toBe(5);
});
```

## 函数别名

基于语义场景使用函数别名，倾向于使用短的函数别名。不使用非语义化的 `Jest` 函数别名，如 `xdescribe` 。

#### 定义

`Jest` 为一系列函数提供了别名以提高可读性。

#### 优势

- 在不同场景下使用不同的函数别名能让代码更接近自然语言。
- 函数别名能缩短代码的长度，方便阅读。

#### 劣势

- 非语义化的函数别名不利于不熟悉 `Jest` 框架的人理解代码。

#### 结论

在函数别名有利于提升代码可读性的情况下，应当使用它们，否则避免使用。

#### 例子

```javascript
// 好
test("drinks returns", () => {
  const drink = jest.fn(() => true);
  drink();
  expect(drink).toReturn();
});

// 不太好：额外的函数名长度，但没有提供有意义的信息
test("drinks returns", () => {
  const drink = jest.fn(() => true);
  drink();
  expect(drink).toHaveReturned();

});

// 好
describe.only("my beverage", () => {
  it("is delicious", () => {
    expect(myBeverage.delicious).toBeTruthy();
  });
});

// 不太好： it is delicious 更接近自然语言书写方式
describe.only("my beverage", () => {
  test("is delicious", () => {
    expect(myBeverage.delicious).toBeTruthy();
  });
});

// 坏：难以理解的名称
fdescribe("my beverage", () => {
  it("is delicious", () => {
    expect(myBeverage.delicious).toBeTruthy();
  });
});
```

## 等待页面

使用 `Puppeteer` 提供的等待方法。

#### 定义

`Puppeteer` 自带了一系列的等待方法来等待页面到达我们需要的状态。

* https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForSelector
* https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForXPath
* https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForFunction
* https://puppeteer.github.io/puppeteer/docs/puppeteer.frame.waitForTimeout

#### 优势

- 这些等待函数提供了查询间隔，超时时长和元素属性等一系列基础功能。
- 条件满足后，对测试的阻塞将很快结束。
- 方便统一调整等待参数。

#### 劣势

- 在这些等待函数内，部分对象无法使用，如 `page` 。
- 比起简单的延时更为复杂。

#### 结论

处于测试的稳定性和性能考虑，倾向于使用 `Puppeteer` 提供的等待方法。

#### 例子

```javascript
// 好
test("net_version should returns 0x5", async () => {
  await page.click("#getChainId");

  await page.waitForFunction(
    () => document.getElementById("chainId").innerText !== "",
  );

  await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe("0x5");
});

// 坏：死板的等待
test("net_version should returns 0x5", async () => {
  await page.click("#getChainId");

  await sleep(1000);

  await expect(page.$eval("#chainId", (e) => e.innerText)).resolves.toBe("0x5");
});
```
