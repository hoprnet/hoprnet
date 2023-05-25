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

const SOME_STATIC_COMP_VALUE = ethers.utils.defaultAbiCoder.encode(
  ["uint256"],
  [123]
);

describe("Scoping", async () => {
  const baseSetup = deployments.createFixture(async () => {
    await deployments.fixture();
    const Avatar = await hre.ethers.getContractFactory("TestAvatar");
    const avatar = await Avatar.deploy();
    const TestContract = await hre.ethers.getContractFactory("TestContract");
    const testContract = await TestContract.deploy();
    return { Avatar, avatar, testContract };
  });

  const setupRolesWithOwnerAndInvoker = deployments.createFixture(async () => {
    const base = await baseSetup();

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
      base.avatar.address,
      base.avatar.address
    );

    await modifier.enableModule(invoker.address);

    return {
      ...base,
      Modifier,
      modifier,
      owner,
      invoker,
    };
  });

  it("scoping one param should work", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    const { data: dataFail } =
      await testContract.populateTransaction.fnWithThreeParams(1, 2, 3);

    const { data: dataOk } =
      await testContract.populateTransaction.fnWithThreeParams(1, 4, 3);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeAllowFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataFail, 0)
    ).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [4])
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataFail, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataOk, 0)
    ).to.not.be.reverted;
  });

  it("unscoping one param should work", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [4])
      );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [5])
      );

    const { data: dataFail } =
      await testContract.populateTransaction.fnWithThreeParams(4, 2, 3);
    const { data: dataOk } =
      await testContract.populateTransaction.fnWithThreeParams(4, 5, 3);

    // fails first
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataFail, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");

    // sanity check
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataOk, 0)
    ).to.not.be.reverted;

    await modifier
      .connect(owner)
      .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 1);

    // works after unscoping
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataFail, 0)
    ).to.not.be.reverted;
  });

  it("scoping one param should work after allow function", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    // this call is supposed to be redudant. This test is checking that scoping one para after scoping all works
    await modifier
      .connect(owner)
      .scopeAllowFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        OPTIONS_NONE
      );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [7])
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 2, 3)
          ).data,
          0
        )
    ).to.be.revertedWith("ParameterNotAllowed()");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(7, 2, 3)
          ).data,
          0
        )
    ).to.not.be.reverted;
  });

  it("scoping one param should work after scope function", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    const { data: dataFail } =
      await testContract.populateTransaction.fnWithThreeParams(1, 2, 3);
    const { data: dataOk } =
      await testContract.populateTransaction.fnWithThreeParams(1, 7, 3);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        [false, true, false],
        [TYPE_STATIC, TYPE_STATIC, TYPE_STATIC],
        [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
        ["0x", ethers.utils.defaultAbiCoder.encode(["uint256"], [7]), "0x"],
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataFail, 0)
    ).to.be.revertedWith("ParameterNotAllowed()");

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, 0, dataOk, 0)
    ).to.not.be.reverted;

    // set last param also as scoped
    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        2,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [8])
      );

    // should account for last param
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 7, 3)
          ).data,
          0
        )
    ).to.be.revertedWith("ParameterNotAllowed()");
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 2, 8)
          ).data,
          0
        )
    ).to.be.revertedWith("ParameterNotAllowed()");
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 7, 8)
          ).data,
          0
        )
    ).to.not.be.reverted;
  });

  it("function scoping all params off is equivalent to allowing function", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        SOME_STATIC_COMP_VALUE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 2, 3)
          ).data,
          0
        )
    ).to.be.revertedWith("ParameterNotAllowed()");

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        [false, false, false],
        [TYPE_STATIC, TYPE_STATIC, TYPE_STATIC],
        [0, 0, 0],
        ["0x", "0x", "0x"],
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 2, 3)
          ).data,
          0
        )
    ).to.emit(testContract, "FnWithThreeParams");
  });

  it("function scoping all params off, including dynamic types, is equivalent to allow function", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithTwoMixedParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeAllowFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithTwoMixedParams(
              false,
              "Hello World!"
            )
          ).data,
          0
        )
    ).to.emit(testContract, "FnWithTwoMixedParams");

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        [false, false],
        [TYPE_STATIC, TYPE_DYNAMIC],
        [0, 0],
        ["0x", "0x"],
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithTwoMixedParams(
              false,
              "Hello World!"
            )
          ).data,
          0
        )
    ).to.emit(testContract, "FnWithTwoMixedParams");
  });

  it("unscoping all params one by one is equivalent to allowFunction", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        [true, true, false],
        [TYPE_STATIC, TYPE_STATIC, TYPE_STATIC],
        [0, 0, 0],
        [
          SOME_STATIC_COMP_VALUE,
          SOME_STATIC_COMP_VALUE,
          SOME_STATIC_COMP_VALUE,
        ],
        OPTIONS_NONE
      );

    await modifier
      .connect(owner)
      .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 0);

    //if some params still scoped returned ParamNotAllowed
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 2, 3)
          ).data,
          0
        )
    ).to.be.revertedWith("ParameterNotAllowed()");

    await modifier
      .connect(owner)
      .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 1);

    //all params off -> FunctionNotAllowed
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithThreeParams(1, 2, 3)
          ).data,
          0
        )
    ).to.be.emit(testContract, "FnWithThreeParams");
  });
  it("unscoping all params one by one, including dynamic types, is equivalent to allowFunction", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithTwoMixedParams")
    );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["bool"], [false])
      );

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        ethers.utils.solidityPack(["string"], ["Hello World!"])
      );

    // should fail because first parameter doesn't comply
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithTwoMixedParams(
              true,
              "Hello World!"
            )
          ).data,
          0
        )
    ).to.be.revertedWith("ParameterNotAllowed()");

    // should work after we unscope first parameter
    await modifier
      .connect(owner)
      .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 0);

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithTwoMixedParams(
              true,
              "Hello World!"
            )
          ).data,
          0
        )
    ).to.emit(testContract, "FnWithTwoMixedParams");

    // unscope second parameter, leaves no parameter scoped
    await modifier
      .connect(owner)
      .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 1);

    // whole function should be not allowed
    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (
            await testContract.populateTransaction.fnWithTwoMixedParams(
              false,
              "Something not previously allowed"
            )
          ).data,
          0
        )
    ).to.emit(testContract, "FnWithTwoMixedParams");
  });

  it("update paramComp should work on already scoped parameter", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();
    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithSingleParam")
    );
    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeAllowFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        OPTIONS_NONE
      );

    const invoke = async (param: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithSingleParam(param))
            .data,
          0
        );

    // sanity
    await expect(invoke(2021)).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_LESS,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [420])
      );

    await expect(invoke(421)).to.be.revertedWith(
      "ParameterGreaterThanAllowed()"
    );
    await expect(invoke(419)).to.not.be.reverted;

    // FLIP THE SAME PARAM to greater
    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_GREATER,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [420])
      );

    await expect(invoke(421)).to.not.be.reverted;
    await expect(invoke(419)).to.be.revertedWith("ParameterLessThanAllowed()");
  });

  it("scoping a high parameter index, after a lower one should work", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;

    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    const invoke = async (a: number, b: number, c: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithThreeParams(a, b, c))
            .data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [1])
      );

    await expect(invoke(1, 3, 2021)).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [2])
      );
    await expect(invoke(1, 3, 2021)).to.be.revertedWith(
      "ParameterNotAllowed()"
    );

    await expect(invoke(1, 2, 3000)).to.not.be.reverted;
  });

  it("scoping a low parameter index, after a higher one should work", async () => {
    const { modifier, testContract, owner, invoker } =
      await setupRolesWithOwnerAndInvoker();

    const ROLE_ID = 0;

    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithThreeParams")
    );

    const invoke = async (a: number, b: number, c: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithThreeParams(a, b, c))
            .data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        2,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [3])
      );

    await expect(invoke(2000, 3000, 3)).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.defaultAbiCoder.encode(["uint256"], [1])
      );
    await expect(invoke(2000, 3000, 3)).to.be.revertedWith(
      "ParameterNotAllowed()"
    );

    await expect(invoke(1, 3000, 3)).to.not.be.reverted;
  });

  describe("Enforces Scope Max Param limit", () => {
    it("checks limit on scopeFunction", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;
      await expect(
        modifier
          .connect(owner)
          .scopeFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            new Array(49).fill(false),
            new Array(49).fill(TYPE_STATIC),
            new Array(49).fill(COMP_EQUAL),
            new Array(49).fill("0x"),
            OPTIONS_NONE
          )
      ).to.be.revertedWith("ScopeMaxParametersExceeded()");

      await expect(
        modifier
          .connect(owner)
          .scopeFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            new Array(48).fill(false),
            new Array(48).fill(TYPE_STATIC),
            new Array(48).fill(0),
            new Array(48).fill("0x"),
            OPTIONS_NONE
          )
      ).to.not.be.reverted;
    });

    it("checks limit on scopeParameter", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;

      await expect(
        modifier
          .connect(owner)
          .scopeParameter(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            48,
            TYPE_DYNAMIC,
            COMP_EQUAL,
            "0x"
          )
      ).to.be.revertedWith("ScopeMaxParametersExceeded()");

      await expect(
        modifier
          .connect(owner)
          .scopeParameter(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            47,
            TYPE_DYNAMIC,
            COMP_EQUAL,
            "0x"
          )
      ).to.not.be.reverted;
    });

    it("checks limit on scopeParameterAsOneOf", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;
      await expect(
        modifier
          .connect(owner)
          .scopeParameterAsOneOf(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            48,
            TYPE_DYNAMIC,
            ["0x", "0x"]
          )
      ).to.be.revertedWith("ScopeMaxParametersExceeded()");

      await expect(
        modifier
          .connect(owner)
          .scopeParameterAsOneOf(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            47,
            TYPE_DYNAMIC,
            ["0x", "0x"]
          )
      ).to.not.be.reverted;
    });

    it("checks limit on unscopeParameter", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;
      await expect(
        modifier
          .connect(owner)
          .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 48)
      ).to.be.revertedWith("ScopeMaxParametersExceeded()");

      await expect(
        modifier
          .connect(owner)
          .unscopeParameter(ROLE_ID, testContract.address, SELECTOR, 47)
      ).to.not.be.reverted;
    });
  });

  describe("Enforces Parameter Size constraints", () => {
    const MORE_THAN_32_BYTES_TEXT =
      "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.";
    const A_32_BYTES_VALUE = ethers.utils.defaultAbiCoder.encode(
      ["uint256"],
      [123]
    );

    it("checks limit on scopeFunction", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;
      const IS_SCOPED = true;

      await expect(
        modifier
          .connect(owner)
          .scopeFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            [IS_SCOPED],
            [TYPE_STATIC],
            [COMP_EQUAL],
            [ethers.utils.solidityPack(["string"], [MORE_THAN_32_BYTES_TEXT])],
            OPTIONS_NONE
          )
      ).to.be.revertedWith("UnsuitableStaticCompValueSize()");

      await expect(
        modifier
          .connect(owner)
          .scopeFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            [IS_SCOPED],
            [TYPE_DYNAMIC32],
            [COMP_EQUAL],
            [ethers.utils.solidityPack(["string"], ["abcdefghijg"])],
            OPTIONS_NONE
          )
      ).to.be.revertedWith("UnsuitableDynamic32CompValueSize()");

      await expect(
        modifier
          .connect(owner)
          .scopeFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            [IS_SCOPED],
            [TYPE_STATIC],
            [COMP_EQUAL],
            [A_32_BYTES_VALUE],
            OPTIONS_NONE
          )
      ).to.be.not.reverted;

      // it doesn't check for unscoped parameter
      await expect(
        modifier
          .connect(owner)
          .scopeFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            [IS_SCOPED, !IS_SCOPED],
            [TYPE_STATIC, TYPE_STATIC],
            [COMP_EQUAL, COMP_EQUAL],
            [
              A_32_BYTES_VALUE,
              ethers.utils.solidityPack(["string"], [MORE_THAN_32_BYTES_TEXT]),
            ],
            OPTIONS_NONE
          )
      ).to.not.be.reverted;
    });

    it("checks limit on scopeParameter", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;
      await expect(
        modifier
          .connect(owner)
          .scopeParameter(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            0,
            TYPE_STATIC,
            COMP_EQUAL,
            ethers.utils.solidityPack(["string"], [MORE_THAN_32_BYTES_TEXT])
          )
      ).to.be.revertedWith("UnsuitableStaticCompValueSize()");

      await expect(
        modifier
          .connect(owner)
          .scopeParameter(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            0,
            TYPE_DYNAMIC32,
            COMP_EQUAL,
            ethers.utils.solidityPack(["string"], ["abcdefghijg"])
          )
      ).to.be.revertedWith("UnsuitableDynamic32CompValueSize()");

      await expect(
        modifier
          .connect(owner)
          .scopeParameter(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            0,
            TYPE_STATIC,
            COMP_EQUAL,
            A_32_BYTES_VALUE
          )
      ).to.not.be.reverted;
    });

    it("checks limit on scopeParameterAsOneOf", async () => {
      const { modifier, testContract, owner } =
        await setupRolesWithOwnerAndInvoker();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("doNothing")
      );

      const ROLE_ID = 0;
      await expect(
        modifier
          .connect(owner)
          .scopeParameterAsOneOf(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            0,
            TYPE_STATIC,
            [
              ethers.utils.solidityPack(["string"], [MORE_THAN_32_BYTES_TEXT]),
              ethers.utils.solidityPack(["string"], [MORE_THAN_32_BYTES_TEXT]),
            ]
          )
      ).to.be.revertedWith("UnsuitableStaticCompValueSize()");

      await expect(
        modifier
          .connect(owner)
          .scopeParameterAsOneOf(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            0,
            TYPE_DYNAMIC32,
            [
              ethers.utils.solidityPack(["string"], ["abcdefghijg"]),
              ethers.utils.solidityPack(["string"], ["abcdefghijg"]),
            ]
          )
      ).to.be.revertedWith("UnsuitableDynamic32CompValueSize()");

      await expect(
        modifier
          .connect(owner)
          .scopeParameterAsOneOf(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            0,
            TYPE_STATIC,
            [A_32_BYTES_VALUE, A_32_BYTES_VALUE]
          )
      ).to.not.be.reverted;
    });
  });
  it("enforces minimum 2 compValues when setting Comparison.OneOf", async () => {
    const { modifier, testContract, owner } =
      await setupRolesWithOwnerAndInvoker();

    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("doNothing")
    );

    const ROLE_ID = 0;
    await expect(
      modifier
        .connect(owner)
        .scopeParameterAsOneOf(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_STATIC,
          []
        )
    ).to.be.revertedWith("NotEnoughCompValuesForOneOf()");

    await expect(
      modifier
        .connect(owner)
        .scopeParameterAsOneOf(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_STATIC,
          [ethers.utils.defaultAbiCoder.encode(["uint256"], [123])]
        )
    ).to.be.revertedWith("NotEnoughCompValuesForOneOf()");

    await expect(
      modifier
        .connect(owner)
        .scopeParameterAsOneOf(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_STATIC,
          [
            ethers.utils.defaultAbiCoder.encode(["uint256"], [123]),
            ethers.utils.defaultAbiCoder.encode(["uint256"], [123]),
          ]
        )
    ).to.not.be.reverted;
  });
});
