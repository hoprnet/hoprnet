import { expect } from "chai";
import hre, { deployments, waffle, ethers } from "hardhat";
import "@nomiclabs/hardhat-ethers";

describe("Comparison", async () => {
  const setup = deployments.createFixture(async () => {
    await deployments.fixture();
    const Avatar = await hre.ethers.getContractFactory("TestAvatar");
    const avatar = await Avatar.deploy();
    const TestContract = await hre.ethers.getContractFactory("TestContract");
    const testContract = await TestContract.deploy();
    const testContractClone = await TestContract.deploy();

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

    return {
      Avatar,
      avatar,
      testContract,
      testContractClone,
      Modifier,
      modifier,
      owner,
      invoker,
    };
  });

  const COMP_EQUAL = 0;
  const COMP_GREATER = 1;
  const COMP_LESS = 2;
  const COMP_ONE_OF = 3;

  const OPTIONS_NONE = 0;
  const OPTIONS_SEND = 1;
  const OPTIONS_DELEGATECALL = 2;
  const OPTIONS_BOTH = 3;

  const TYPE_STATIC = 0;
  const TYPE_DYNAMIC = 1;
  const TYPE_DYNAMIC32 = 2;

  it("scopeFunction throws on input length mistmatch", async () => {
    const { modifier, testContract, owner } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithTwoMixedParams")
    );

    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [true, false],
          [TYPE_STATIC, TYPE_DYNAMIC, TYPE_DYNAMIC],
          [COMP_EQUAL, COMP_EQUAL],
          ["0x", "0x"],
          OPTIONS_NONE
        )
    ).to.be.revertedWith("ArraysDifferentLength()");

    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [true, false],
          [TYPE_STATIC, TYPE_DYNAMIC],
          [COMP_EQUAL, COMP_EQUAL, COMP_EQUAL],
          ["0x", "0x"],
          OPTIONS_NONE
        )
    ).to.be.revertedWith("ArraysDifferentLength()");

    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [true, false],
          [TYPE_STATIC, TYPE_DYNAMIC],
          [COMP_EQUAL, COMP_EQUAL],
          ["0x", "0x", "0x"],
          OPTIONS_NONE
        )
    ).to.be.revertedWith("ArraysDifferentLength()");

    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [true, false],
          [TYPE_STATIC, TYPE_DYNAMIC],
          [COMP_EQUAL, COMP_EQUAL],
          [ethers.utils.defaultAbiCoder.encode(["bool"], [false]), "0x"],
          OPTIONS_NONE
        )
    ).to.not.be.reverted;
  });

  it("enforces paramComp for scopeFunction", async () => {
    const { modifier, testContract, owner } = await setup();

    const IS_SCOPED = true;
    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("doNothing")
    );

    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [!IS_SCOPED, IS_SCOPED, !IS_SCOPED],
          [TYPE_STATIC, TYPE_STATIC, TYPE_STATIC],
          [COMP_EQUAL, COMP_ONE_OF, COMP_EQUAL],
          [
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
          ],
          OPTIONS_NONE
        )
    ).to.be.revertedWith("UnsuitableOneOfComparison");

    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [!IS_SCOPED, IS_SCOPED, !IS_SCOPED],
          [TYPE_STATIC, TYPE_DYNAMIC, TYPE_STATIC],
          [COMP_EQUAL, COMP_GREATER, COMP_GREATER],
          [
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
            "0x",
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
          ],
          OPTIONS_NONE
        )
    ).to.be.revertedWith("UnsuitableRelativeComparison");

    // for greater
    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [!IS_SCOPED, IS_SCOPED, IS_SCOPED],
          [TYPE_STATIC, TYPE_DYNAMIC, TYPE_STATIC],
          [COMP_EQUAL, COMP_EQUAL, COMP_GREATER],
          [
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
            "0x",
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
          ],
          OPTIONS_NONE
        )
    ).to.be.not.be.reverted;

    // for less
    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          [!IS_SCOPED, IS_SCOPED, IS_SCOPED],
          [TYPE_STATIC, TYPE_DYNAMIC, TYPE_STATIC],
          [COMP_EQUAL, COMP_EQUAL, COMP_LESS],
          [
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
            "0x",
            ethers.utils.defaultAbiCoder.encode(["bool"], [false]),
          ],
          OPTIONS_NONE
        )
    ).to.not.be.reverted;
  });
  it("enforces paramComp for scopeParameter", async () => {
    const { modifier, testContract, owner } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("doNothing")
    );

    await expect(
      modifier
        .connect(owner)
        .scopeParameter(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_DYNAMIC,
          COMP_ONE_OF,
          "0x"
        )
    ).to.be.revertedWith("UnsuitableOneOfComparison");

    await expect(
      modifier
        .connect(owner)
        .scopeParameter(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_DYNAMIC,
          COMP_GREATER,
          "0x"
        )
    ).to.be.revertedWith("UnsuitableRelativeComparison");

    await expect(
      modifier
        .connect(owner)
        .scopeParameter(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_DYNAMIC,
          COMP_EQUAL,
          "0x"
        )
    ).to.not.be.reverted;

    await expect(
      modifier
        .connect(owner)
        .scopeParameter(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_STATIC,
          COMP_ONE_OF,
          ethers.utils.defaultAbiCoder.encode(["uint256"], [123])
        )
    ).to.be.revertedWith("UnsuitableOneOfComparison");

    await expect(
      modifier
        .connect(owner)
        .scopeParameter(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          0,
          TYPE_STATIC,
          COMP_GREATER,
          ethers.utils.defaultAbiCoder.encode(["uint256"], [123])
        )
    ).to.not.be.reverted;

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
          ethers.utils.defaultAbiCoder.encode(["uint256"], [123])
        )
    ).to.not.be.reverted;
  });

  it("passes an eq comparison", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithSingleParam")
    );

    const invoke = async (a: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithSingleParam(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
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
        ethers.utils.solidityPack(["uint256"], [123])
      );

    await expect(invoke(321)).to.be.revertedWith("ParameterNotAllowed()");
    await expect(invoke(123)).to.not.be.reverted;
  });
  it("passes an eq comparison for dynamic", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithTwoMixedParams")
    );

    const invoke = async (a: boolean, b: string) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithTwoMixedParams(a, b))
            .data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        ethers.utils.solidityPack(["string"], ["Some string"])
      );

    await expect(invoke(false, "Some string")).to.not.be.reverted;
    await expect(invoke(false, "Some other string")).to.be.revertedWith(
      "ParameterNotAllowed()"
    );
  });

  it("passes an eq comparison for dynamic - empty buffer", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("dynamic")
    );

    const invoke = async (a: any) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.dynamic(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        "0x"
      );

    await expect(invoke("0x")).to.not.be.reverted;
    await expect(invoke("0x12")).to.be.revertedWith("ParameterNotAllowed()");
  });

  it("passes an eq comparison for dynamic32", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("dynamicDynamic32")
    );

    const invoke = async (a: string, b: any[]) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.dynamicDynamic32(a, b)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC32,
        COMP_EQUAL,
        ethers.utils.solidityPack(["bytes2[]"], [["0x1234", "0xabcd"]])
      );

    //longer
    await expect(
      invoke("Doesn't matter", ["0x1234", "0xabcd", "0xabcd"])
    ).to.be.revertedWith("ParameterNotAllowed()");

    //shorter
    await expect(invoke("Doesn't matter", ["0x1234"])).to.be.revertedWith(
      "ParameterNotAllowed()"
    );

    // different
    await expect(
      invoke("Doesn't matter", ["0x0234", "0xabcd"])
    ).to.be.revertedWith("ParameterNotAllowed()");

    await expect(invoke("Doesn't matter", ["0x1234", "0xabcd"])).to.not.be
      .reverted;
  });

  it("passes an eq comparison for dynamic32 - empty array", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("dynamic32")
    );

    const invoke = async (a: any) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.dynamic32(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_DYNAMIC32,
        COMP_EQUAL,
        []
      );

    await expect(invoke([])).to.not.be.reverted;
    await expect(invoke(["0xaabbccddeeff0011"])).to.be.revertedWith(
      "ParameterNotAllowed()"
    );
  });

  it("re-scopes an eq paramComp", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithSingleParam")
    );

    const invoke = async (a: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithSingleParam(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
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
        ethers.utils.solidityPack(["uint256"], [123])
      );

    await expect(invoke(321)).to.be.revertedWith("ParameterNotAllowed()");
    await expect(invoke(123)).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_GREATER,
        ethers.utils.solidityPack(["uint256"], [123])
      );

    await expect(invoke(123)).to.be.revertedWith("ParameterLessThanAllowed()");
    await expect(invoke(124)).to.not.be.reverted;
  });

  it("passes a oneOf comparison", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithSingleParam")
    );

    const invoke = async (a: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithSingleParam(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameterAsOneOf(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        COMP_EQUAL,
        [
          ethers.utils.defaultAbiCoder.encode(["uint256"], [11]),
          ethers.utils.defaultAbiCoder.encode(["uint256"], [22]),
        ]
      );

    await expect(invoke(11)).to.not.be.reverted;
    await expect(invoke(22)).to.not.be.reverted;
    await expect(invoke(33)).to.be.revertedWith("ParameterNotOneOfAllowed()");
  });

  it("passes a oneOf comparison for dynamic", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithTwoMixedParams")
    );

    const invoke = async (a: boolean, b: string) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithTwoMixedParams(a, b))
            .data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameterAsOneOf(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC,
        [
          ethers.utils.solidityPack(["string"], ["Hello World!"]),
          ethers.utils.solidityPack(["string"], ["Good Morning!"]),
          ethers.utils.solidityPack(["string"], ["gm!!!!!!!!!!!"]),
        ]
      );

    await expect(invoke(true, "Hello World!")).to.not.be.reverted;
    await expect(invoke(false, "Good Morning!")).to.not.be.reverted;
    await expect(invoke(true, "gm!!!!!!!!!!!")).to.not.be.reverted;

    await expect(invoke(false, "Something else")).to.be.revertedWith(
      "ParameterNotOneOfAllowed()"
    );
  });

  it("passes a oneOf comparison for dynamic32", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("dynamicDynamic32")
    );

    const invoke = async (a: string, b: any) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.dynamicDynamic32(a, b)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameterAsOneOf(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        1,
        TYPE_DYNAMIC32,
        [
          ethers.utils.solidityPack(["bytes2[]"], [["0x1111", "0x1111"]]),
          ethers.utils.solidityPack(["bytes2[]"], [["0xffff", "0xffff"]]),
        ]
      );

    await expect(invoke("A", ["0x1111", "0x1111"])).to.not.be.reverted;
    await expect(invoke("B", ["0xffff", "0xffff"])).to.not.be.reverted;

    await expect(
      invoke("C", ["0x1111", "0x1111", "0x1234"])
    ).to.be.revertedWith("ParameterNotOneOfAllowed()");
    await expect(
      invoke("D", ["0xffff", "0x1111", "0x1111"])
    ).to.be.revertedWith("ParameterNotOneOfAllowed()");
    await expect(invoke("E", ["0x1111"])).to.be.revertedWith(
      "ParameterNotOneOfAllowed()"
    );
    await expect(invoke("F", ["0xf111", "0x1111"])).to.be.revertedWith(
      "ParameterNotOneOfAllowed()"
    );
    await expect(invoke("G", ["0x1111", "0x111f"])).to.be.revertedWith(
      "ParameterNotOneOfAllowed()"
    );
    await expect(invoke("H", [])).to.be.revertedWith(
      "ParameterNotOneOfAllowed()"
    );
  });

  it("re-scopes a oneOf comparison to simple paramComp", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithSingleParam")
    );

    const invoke = async (a: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithSingleParam(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameterAsOneOf(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        [
          ethers.utils.solidityPack(["uint256"], [501]),
          ethers.utils.solidityPack(["uint256"], [602]),
          ethers.utils.solidityPack(["uint256"], [703]),
        ]
      );

    await expect(invoke(501)).to.not.be.reverted;
    await expect(invoke(602)).to.not.be.reverted;
    await expect(invoke(703)).to.not.be.reverted;
    await expect(invoke(123)).to.be.revertedWith("ParameterNotOneOfAllowed()");

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_EQUAL,
        ethers.utils.solidityPack(["uint256"], [123])
      );

    await expect(invoke(602)).to.be.revertedWith("ParameterNotAllowed()");
    await expect(invoke(123)).to.not.be.reverted;
  });

  it("should pass a gt/lt comparison", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const ROLE_ID = 0;
    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("fnWithSingleParam")
    );

    const invoke = async (a: number) =>
      modifier
        .connect(invoker)
        .execTransactionFromModule(
          testContract.address,
          0,
          (await testContract.populateTransaction.fnWithSingleParam(a)).data,
          0
        );

    await modifier
      .connect(owner)
      .assignRoles(invoker.address, [ROLE_ID], [true]);

    // set it to true
    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_GREATER,
        ethers.utils.solidityPack(["uint256"], [1234])
      );

    await expect(invoke(1233)).to.be.revertedWith("ParameterLessThanAllowed()");
    await expect(invoke(1234)).to.be.revertedWith("ParameterLessThanAllowed()");
    await expect(invoke(1235)).to.not.be.reverted;

    await modifier
      .connect(owner)
      .scopeParameter(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        0,
        TYPE_STATIC,
        COMP_LESS,
        ethers.utils.solidityPack(["uint256"], [2345])
      );

    await expect(invoke(2346)).to.be.revertedWith(
      "ParameterGreaterThanAllowed()"
    );
    await expect(invoke(2345)).to.be.revertedWith(
      "ParameterGreaterThanAllowed()"
    );
    await expect(invoke(2344)).to.not.be.reverted;
  });
});
