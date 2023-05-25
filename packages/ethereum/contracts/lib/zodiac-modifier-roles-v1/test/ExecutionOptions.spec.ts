import { expect } from "chai";
import hre, { deployments, waffle, ethers } from "hardhat";
import "@nomiclabs/hardhat-ethers";

const OPTIONS_NONE = 0;
const OPTIONS_SEND = 1;
const OPTIONS_DELEGATECALL = 2;
const OPTIONS_BOTH = 3;

const ROLE_ID = 0;

describe("ExecutionOptions", async () => {
  const setup = deployments.createFixture(async () => {
    await deployments.fixture();
    const Avatar = await hre.ethers.getContractFactory("TestAvatar");
    const avatar = await Avatar.deploy();
    const TestContract = await hre.ethers.getContractFactory("TestContract");
    const testContract = await TestContract.deploy();

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

    // fund avatar
    await invoker.sendTransaction({
      to: avatar.address,
      value: ethers.utils.parseEther("10"),
    });

    return {
      Avatar,
      avatar,
      testContract,
      Modifier,
      modifier,
      owner,
      invoker,
    };
  });

  describe("sending eth", () => {
    describe("target allowed - aka Clearance.TARGET", () => {
      it("ExecutionOptions.NONE - FAILS sending eth to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_NONE);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });

      it("ExecutionOptions.NONE - FAILS sending eth to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_NONE);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });

      it("ExecutionOptions.SEND - OK sending eth to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_SEND);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        )
          .to.be.emit(testContract, "ReceiveEthAndDoNothing")
          .withArgs(value);
      });

      it("ExecutionOptions.SEND - OK sending eth to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_SEND);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        )
          .to.be.emit(testContract, "ReceiveFallback")
          .withArgs(value);
      });

      it("ExecutionOptions.DELEGATECALL - FAILS sending ETH to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_DELEGATECALL);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });
      it("ExecutionOptions.DELEGATECALL - FAILS sending ETH to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_DELEGATECALL);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });
      it("ExecutionOptions.BOTH - OK sending ETH to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_BOTH);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        )
          .to.be.emit(testContract, "ReceiveEthAndDoNothing")
          .withArgs(value);
      });

      it("ExecutionOptions.BOTH - OK sending ETH to fallback function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .allowTarget(ROLE_ID, testContract.address, OPTIONS_BOTH);

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        )
          .to.be.emit(testContract, "ReceiveEthAndDoNothing")
          .withArgs(value);
      });
    });

    describe("target allowed partially - aka Clearance.FUNCTION", () => {
      it("ExecutionOptions.NONE - FAILS sending eth to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const SELECTOR = testContract.interface.getSighash(
          testContract.interface.getFunction("receiveEthAndDoNothing")
        );

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

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
            .execTransactionFromModule(testContract.address, value, data, 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });

      it("ExecutionOptions.NONE - FAILS sending eth to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            "0x00000000",
            OPTIONS_NONE
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });

      it("ExecutionOptions.SEND - OK sending eth to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1.123");

        const SELECTOR = testContract.interface.getSighash(
          testContract.interface.getFunction("receiveEthAndDoNothing")
        );

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            OPTIONS_SEND
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        )
          .to.be.emit(testContract, "ReceiveEthAndDoNothing")
          .withArgs(value);
      });

      it("ExecutionOptions.SEND - OK sending eth to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1.123");
        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            "0x00000000",
            OPTIONS_SEND
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        )
          .to.be.emit(testContract, "ReceiveFallback")
          .withArgs(value);
      });
      it("ExecutionOptions.SEND - only updating options is not an allowance", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1.123");

        const SELECTOR = testContract.interface.getSighash(
          testContract.interface.getFunction("receiveEthAndDoNothing")
        );

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        // missing scopeTarget

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            OPTIONS_SEND
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        ).to.be.revertedWith("TargetAddressNotAllowed()");
      });

      it("ExecutionOptions.DELEGATECALL - FAILS sending ETH to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        const SELECTOR = testContract.interface.getSighash(
          testContract.interface.getFunction("receiveEthAndDoNothing")
        );

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            OPTIONS_DELEGATECALL
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });
      it("ExecutionOptions.DELEGATECALL - FAILS sending ETH to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1");

        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            "0x00000000",
            OPTIONS_DELEGATECALL
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        ).to.be.revertedWith("SendNotAllowed()");
      });

      it("ExecutionOptions.BOTH - OK sending eth to payable function", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1.123");

        const SELECTOR = testContract.interface.getSighash(
          testContract.interface.getFunction("receiveEthAndDoNothing")
        );

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            OPTIONS_BOTH
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        )
          .to.be.emit(testContract, "ReceiveEthAndDoNothing")
          .withArgs(value);
      });

      it("ExecutionOptions.BOTH - OK sending eth to fallback", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1.123");
        await modifier
          .connect(owner)
          .scopeTarget(ROLE_ID, testContract.address);

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            "0x00000000",
            OPTIONS_BOTH
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, "0x", 0)
        )
          .to.be.emit(testContract, "ReceiveFallback")
          .withArgs(value);
      });
      it("ExecutionOptions.BOTH - only updating options is not an allowance", async () => {
        const { modifier, testContract, owner, invoker } = await setup();

        const value = ethers.utils.parseEther("1.123");

        const SELECTOR = testContract.interface.getSighash(
          testContract.interface.getFunction("receiveEthAndDoNothing")
        );

        const { data } =
          await testContract.populateTransaction.receiveEthAndDoNothing();

        // missing scopeTarget

        await modifier
          .connect(owner)
          .scopeAllowFunction(
            ROLE_ID,
            testContract.address,
            SELECTOR,
            OPTIONS_BOTH
          );

        await expect(
          modifier
            .connect(invoker)
            .execTransactionFromModule(testContract.address, value, data, 0)
        ).to.be.revertedWith("TargetAddressNotAllowed()");
      });
    });
  });

  describe("delegatecall", () => {
    it("target allowed - can delegatecall", async () => {
      const { modifier, testContract, owner, invoker } = await setup();

      const { data } = await testContract.populateTransaction.emitTheSender();

      await modifier
        .connect(owner)
        .allowTarget(ROLE_ID, testContract.address, OPTIONS_DELEGATECALL);

      await expect(
        modifier
          .connect(invoker)
          .execTransactionFromModule(testContract.address, 0, data, 1)
      ).to.not.be.reverted;
    });
    it("target allowed - cannot delegatecall", async () => {
      const { modifier, testContract, owner, invoker } = await setup();

      const { data } = await testContract.populateTransaction.emitTheSender();

      await modifier
        .connect(owner)
        .allowTarget(ROLE_ID, testContract.address, OPTIONS_NONE);

      await expect(
        modifier
          .connect(invoker)
          .execTransactionFromModule(testContract.address, 0, data, 1)
      ).to.be.revertedWith("DelegateCallNotAllowed()");
    });
    it("target partially allowed - can delegatecall", async () => {
      const { modifier, testContract, owner, invoker } = await setup();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("emitTheSender")
      );

      const { data } = await testContract.populateTransaction.emitTheSender();

      await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

      await modifier
        .connect(owner)
        .scopeAllowFunction(
          ROLE_ID,
          testContract.address,
          SELECTOR,
          OPTIONS_BOTH
        );

      await expect(
        modifier
          .connect(invoker)
          .execTransactionFromModule(testContract.address, 0, data, 1)
      ).not.to.be.reverted;
    });

    it("target partially allowed - cannot delegatecall", async () => {
      const { modifier, testContract, owner, invoker } = await setup();

      const SELECTOR = testContract.interface.getSighash(
        testContract.interface.getFunction("emitTheSender")
      );

      const { data } = await testContract.populateTransaction.emitTheSender();

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
          .execTransactionFromModule(testContract.address, 0, data, 1)
      ).to.be.revertedWith("DelegateCallNotAllowed()");
    });
  });

  it("ExecutionOptions can be set independently", async () => {
    const { modifier, testContract, owner, invoker } = await setup();

    const value = ethers.utils.parseEther("1.123");

    const SELECTOR = testContract.interface.getSighash(
      testContract.interface.getFunction("receiveEthAndDoNothing")
    );

    const { data } =
      await testContract.populateTransaction.receiveEthAndDoNothing();

    await modifier.connect(owner).scopeTarget(ROLE_ID, testContract.address);

    await modifier
      .connect(owner)
      .scopeAllowFunction(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        OPTIONS_SEND
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, value, data, 0)
    )
      .to.be.emit(testContract, "ReceiveEthAndDoNothing")
      .withArgs(value);

    await modifier
      .connect(owner)
      .scopeFunctionExecutionOptions(
        ROLE_ID,
        testContract.address,
        SELECTOR,
        OPTIONS_NONE
      );

    await expect(
      modifier
        .connect(invoker)
        .execTransactionFromModule(testContract.address, value, data, 0)
    ).to.be.revertedWith("SendNotAllowed");
  });
});
