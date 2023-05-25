import { AddressOne } from "@gnosis.pm/safe-contracts";
import { expect } from "chai";
import hre, { deployments, waffle } from "hardhat";
import "@nomiclabs/hardhat-ethers";

const ROLE_ID = 123;

const COMP_EQUAL = 0;

const OPTIONS_NONE = 0;
const OPTIONS_SEND = 1;
const OPTIONS_DELEGATECALL = 2;
const OPTIONS_BOTH = 3;

const TYPE_STATIC = 0;
const TYPE_DYNAMIC = 1;
const TYPE_DYNAMIC32 = 2;

// Pending: https://github.com/EthWorks/Waffle/issues/609

describe.skip("EmitsEvent", async () => {
  const setup = deployments.createFixture(async () => {
    await deployments.fixture();
    const Avatar = await hre.ethers.getContractFactory("TestAvatar");
    const avatar = await Avatar.deploy();

    const [owner] = waffle.provider.getWallets();

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

    return {
      Avatar,
      avatar,
      modifier,
      owner,
    };
  });

  it("AllowTarget", async () => {
    const { owner, modifier } = await setup();

    await expect(
      modifier.connect(owner).allowTarget(ROLE_ID, AddressOne, OPTIONS_SEND)
    )
      .to.emit(modifier, "AllowTarget")
      .withArgs(ROLE_ID, AddressOne, OPTIONS_SEND);
  });
  it("ScopeTarget", async () => {
    const { owner, modifier } = await setup();

    await expect(modifier.connect(owner).scopeTarget(ROLE_ID, AddressOne))
      .to.emit(modifier, "ScopeTarget")
      .withArgs(ROLE_ID, AddressOne);
  });
  it("RevokeTarget", async () => {
    const { owner, modifier } = await setup();

    await expect(modifier.connect(owner).revokeTarget(ROLE_ID, AddressOne))
      .to.emit(modifier, "RevokeTarget")
      .withArgs(ROLE_ID, AddressOne);
  });
  it("ScopeAllowFunction", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .scopeAllowFunction(ROLE_ID, AddressOne, "0x12345678", OPTIONS_BOTH)
    )
      .to.emit(modifier, "ScopeAllowFunction")
      .withArgs(ROLE_ID, AddressOne, "0x12345678", OPTIONS_BOTH);
  });
  it("ScopeRevokeFunction", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .scopeRevokeFunction(ROLE_ID, AddressOne, "0x12345678")
    )
      .to.emit(modifier, "ScopeRevokeFunction")
      .withArgs(ROLE_ID, AddressOne, "0x12345678");
  });
  it("ScopeFunction", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .scopeFunction(
          ROLE_ID,
          AddressOne,
          "0x12345678",
          [],
          [],
          [],
          [],
          OPTIONS_NONE
        )
    )
      .to.emit(modifier, "ScopeFunction")
      .withArgs(
        ROLE_ID,
        AddressOne,
        "0x12345678",
        [],
        [],
        [],
        [],
        OPTIONS_NONE
      );
  });
  it("ScopeParameter", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .scopeParameter(
          ROLE_ID,
          AddressOne,
          "0x12345678",
          0,
          TYPE_DYNAMIC,
          COMP_EQUAL,
          "0x"
        )
    )
      .to.emit(modifier, "ScopeParameter")
      .withArgs(
        ROLE_ID,
        AddressOne,
        "0x12345678",
        0,
        TYPE_DYNAMIC,
        COMP_EQUAL,
        "0x"
      );
  });
  it("ScopeParameterAsOneOf", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .scopeParameterAsOneOf(
          ROLE_ID,
          AddressOne,
          "0x12345678",
          0,
          TYPE_DYNAMIC,
          []
        )
    )
      .to.emit(modifier, "ScopeParameterAsOneOf")
      .withArgs(ROLE_ID, AddressOne, "0x12345678", 0, TYPE_DYNAMIC, []);
  });
  it("UnscopeParameter", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .unscopeParameter(ROLE_ID, AddressOne, "0x12345678", 0)
    )
      .to.emit(modifier, "UnscopeParameter")
      .withArgs(ROLE_ID, AddressOne, "0x12345678", 0);
  });

  it("ScopeFunctionExecutionOptions", async () => {
    const { modifier, owner } = await setup();
    await expect(
      modifier
        .connect(owner)
        .scopeFunctionExecutionOptions(
          ROLE_ID,
          AddressOne,
          "0x12345678",
          OPTIONS_SEND
        )
    )
      .to.emit(modifier, "ScopeFunctionExecutionOptions")
      .withArgs(ROLE_ID, AddressOne, "0x12345678", OPTIONS_SEND);
  });
});
