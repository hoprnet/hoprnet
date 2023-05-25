import { AddressOne } from "@gnosis.pm/safe-contracts";
import { expect } from "chai";
import hre, { deployments, waffle, ethers } from "hardhat";
import "@nomiclabs/hardhat-ethers";

const COMP_EQUAL = 0;
const COMP_GREATER = 1;
const COMP_LESS = 2;

const OPTIONS_NONE = 0;
const OPTIONS_SEND = 1;
const OPTIONS_DELEGATECALL = 2;
const OPTIONS_BOTH = 3;

const TYPE_STATIC = 0;
const TYPE_DYNAMIC = 1;
const TYPE_DYNAMIC32 = 2;

describe("PluckParam - Decoding", async () => {
  const ROLE_ID = 0;
  const setup = deployments.createFixture(async () => {
    await deployments.fixture();
    const Avatar = await hre.ethers.getContractFactory("TestAvatar");
    const avatar = await Avatar.deploy();

    const TestPluckParam = await hre.ethers.getContractFactory(
      "TestPluckParam"
    );
    const testPluckParam = await TestPluckParam.deploy();

    const [owner, invoker] = waffle.provider.getWallets();

    const Permissions = await hre.ethers.getContractFactory("Permissions");
    const permissions = await Permissions.deploy();
    const Modifier = await hre.ethers.getContractFactory("Roles", {
      libraries: {
        Permissions: permissions.address,
      },
    });

    const modifier = await Modifier.deploy(
      owner.address,
      avatar.address,
      avatar.address
    );

    await modifier.enableModule(invoker.address);

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testPluckParam.address);

    return {
      testPluckParam,
      Modifier,
      modifier,
      owner,
      invoker,
    };
  });

  it("static, dynamic - (bytes4, string)", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticDynamic")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true],
        [TYPE_STATIC, TYPE_DYNAMIC],
        [COMP_EQUAL, COMP_EQUAL],
        [
          encodeStatic(["bytes4"], ["0x12345678"]),
          encodeDynamic(["string"], ["Hello World!"]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.staticDynamic(
        "0x12345678",
        "Hello World!"
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.staticDynamic(
        "0x12345678",
        "Good Morning!"
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "StaticDynamic");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });

  it("static, dynamic, dynamic32 - (address,bytes,uint32[])", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticDynamicDynamic32")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_STATIC, TYPE_DYNAMIC, TYPE_DYNAMIC32],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeStatic(["address"], [AddressOne]),
          encodeDynamic(["bytes"], ["0xabcd"]),
          encodeDynamic32(["uint32[]"], [[1, 2, 3]]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.staticDynamicDynamic32(
        AddressOne,
        "0xabcd",
        [1, 2, 3]
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.staticDynamicDynamic32(
        AddressOne,
        "0xabcd",
        [1, 2, 4]
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "StaticDynamicDynamic32");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });
  it("static, dynamic32, dynamic - (uint32,bytes4[],string)", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticDynamic32Dynamic")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_STATIC, TYPE_DYNAMIC32, TYPE_DYNAMIC],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeStatic(["uint32"], [123]),
          encodeDynamic32(["bytes4[]"], [["0xabcdef12"]]),
          encodeDynamic(["string"], ["Hello World!"]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.staticDynamic32Dynamic(
        [123],
        ["0xabcdef12"],
        "Hello World!"
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.staticDynamic32Dynamic(
        [123],
        ["0xabcdef12"],
        "Hello World?"
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "StaticDynamic32Dynamic");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });

  it("dynamic, static, dynamic32 - (bytes,bool,bytes2[])", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("dynamicStaticDynamic32")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_DYNAMIC, TYPE_STATIC, TYPE_DYNAMIC32],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeDynamic(["bytes"], ["0x12ab45"]),
          encodeStatic(["bool"], [false]),
          encodeDynamic32(["bytes2[]"], [["0x1122", "0x3344"]]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.dynamicStaticDynamic32(
        "0x12ab45",
        false,
        ["0x1122", "0x3344"]
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.dynamicStaticDynamic32(
        "0x12ab45",
        false,
        ["0x1122", "0x3344", "0x5566"]
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "DynamicStaticDynamic32");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });
  it("dynamic, dynamic32, static - (string,uint32[],uint256)", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("dynamicDynamic32Static")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_DYNAMIC, TYPE_DYNAMIC32, TYPE_STATIC],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeDynamic(["string"], ["Hello World!"]),
          encodeDynamic32(["uint32[]"], [[1975, 2000, 2025]]),
          encodeStatic(["uint256"], [123456789]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.dynamicDynamic32Static(
        "Hello World!",
        [1975, 2000, 2025],
        123456789
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.dynamicDynamic32Static(
        "Hello World!",
        [1975, 2000],
        123456789
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "DynamicDynamic32Static");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });
  it("dynamic32, static, dynamic - (address[],bytes2,bytes)", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("dynamic32StaticDynamic")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_DYNAMIC32, TYPE_STATIC, TYPE_DYNAMIC],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeDynamic32(["address[]"], [[AddressOne, AddressOne]]),
          encodeStatic(["bytes2"], ["0xaabb"]),
          encodeDynamic(["bytes"], ["0x0123456789abcdef"]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.dynamic32StaticDynamic(
        [AddressOne, AddressOne],
        "0xaabb",
        "0x0123456789abcdef"
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.dynamic32StaticDynamic(
        [AddressOne, AddressOne],
        "0xaabb",
        "0x0123456789abcdef0123456789abcdef"
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "Dynamic32StaticDynamic");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });
  it("dynamic32, dynamic, static - (bytes2[],string,uint32)", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("dynamic32DynamicStatic")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_DYNAMIC32, TYPE_DYNAMIC, TYPE_STATIC],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeDynamic32(["bytes2[]"], [["0xaabb", "0xccdd", "0x1122"]]),
          encodeDynamic(["string"], ["Hello World!"]),
          encodeStatic(["uint32"], [8976]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.dynamic32DynamicStatic(
        ["0xaabb", "0xccdd", "0x1122"],
        "Hello World!",
        8976
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.dynamic32DynamicStatic(
        ["0xaabb", "0xccdd", "0x3344"],
        "Hello World!",
        8976
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "Dynamic32DynamicStatic");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });

  it("don't try this at home", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("unsupportedFixedSizeAndDynamic")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true, true, true],
        [TYPE_STATIC, TYPE_STATIC, TYPE_DYNAMIC],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        [
          encodeStatic(["bool"], [false]),
          encodeStatic(["bool"], [false]),
          encodeDynamic(["string"], ["Hello World!"]),
        ],
        OPTIONS_NONE
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.unsupportedFixedSizeAndDynamic(
        [false, false],
        "Hello World!"
      );

    const { data: dataBad } =
      await testPluckParam.populateTransaction.unsupportedFixedSizeAndDynamic(
        [true, false],
        "Hello World!"
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.emit(testPluckParam, "UnsupportedFixedSizeAndDynamic");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");
  });

  it("static - fails if calldata is too short", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticFn")
    );

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        [true],
        [TYPE_STATIC],
        [COMP_EQUAL],
        [encodeStatic(["bytes4"], ["0x12345678"])],
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, SELECTOR, 0)
    ).to.be.revertedWith("CalldataOutOfBounds()");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testPluckParam.address,
          0,
          `${SELECTOR}aabbccdd`,
          0
        )
    ).to.be.revertedWith("CalldataOutOfBounds()");
  });

  it("static - fails with param scoped out of bounds", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticFn")
    );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        encodeStatic(["bytes4"], ["0x12345678"])
      );

    const { data } = await testPluckParam.populateTransaction.staticFn(
      "0x12345678"
    );

    // ok
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, data, 0)
    ).to.emit(testPluckParam, "Static");

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        1,
        TYPE_STATIC,
        COMP_EQUAL,
        encodeStatic(["bytes4"], ["0x12345678"])
      );

    // ngmi
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, data, 0)
    ).to.be.revertedWith("CalldataOutOfBounds()");
  });

  it("dynamic - fails if calldata too short", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticDynamic")
    );

    const SELECTOR_OTHER = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticFn")
    );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        encodeDynamic(["string"], ["Hello World!"])
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.staticDynamic(
        "0x12345678",
        "Hello World!"
      );

    const dataShort = (
      (await testPluckParam.populateTransaction.staticFn("0x12345678"))
        .data as string
    ).replace(SELECTOR_OTHER.slice(2), SELECTOR.slice(2));

    // shortned call
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataShort, 0)
    ).to.be.revertedWith("CalldataOutOfBounds()");

    // just the selector
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, SELECTOR, 0)
    ).to.be.revertedWith("CalldataOutOfBounds()");

    // ok
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.not.be.reverted;
  });

  it("dynamic - fails if payload is missing", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticDynamic")
    );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        encodeDynamic(["string"], ["Hello World!"])
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.staticDynamic(
        "0x12345678",
        "Hello World!"
      );

    // 0x737c0619 -> staticDynamic selector
    const dataBad = `\
0x737c0619\
0000000000000000000000000000000000000000000000000000000012345678\
0000000000000000000000000000000000000000000000000000000000300001`;

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataBad, 0)
    ).to.be.revertedWith("CalldataOutOfBounds()");

    // ok
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.not.be.reverted;
  });

  it("dynamic - fails with parameter scoped out of bounds", async () => {
    const { modifier, testPluckParam, owner, invoker } = await setup();

    const SELECTOR = testPluckParam.interface.getSighash(
      testPluckParam.interface.getFunction("staticDynamic")
    );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        encodeDynamic(["string"], ["Hello World!"])
      );

    const { data: dataGood } =
      await testPluckParam.populateTransaction.staticDynamic(
        "0x12345678",
        "Hello World!"
      );

    // ok
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testPluckParam.address,
        SELECTOR,
        15,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        encodeDynamic(["string"], ["Hello World!"])
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testPluckParam.address, 0, dataGood, 0)
    ).to.be.revertedWith("CalldataOutOfBounds()");
  });
});

function splitAndPrint(s: string | undefined) {
  s = s as string;
  s = (s as string).slice(2);
  while (s.length) {
    console.log(s.slice(0, 64));
    s = s.slice(64);
  }
}

function encodeStatic(types: any[], values: any[]) {
  return ethers.utils.defaultAbiCoder.encode(types, values);
}

function encodeDynamic(types: any[], values: any[]) {
  return ethers.utils.solidityPack(types, values);
}

function encodeDynamic32(types: any[], values: any[]) {
  return ethers.utils.solidityPack(types, values);
}
