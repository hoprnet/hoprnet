import BN = require("bn.js");
import {
  PaymentChannelContract,
  PaymentChannelInstance,
  HoprTokenContract,
  HoprTokenInstance
} from "../../../types/truffle-contracts";
import { signPayment, recoverSigner } from "./utils";
import { PromiseType } from "../../../types/typescript";
import { time, expectEvent } from "@openzeppelin/test-helpers";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");

const PaymentChannel: PaymentChannelContract = artifacts.require(
  "PaymentChannel"
);

// taken from "scripts/test.sh"
const senderPrivKey =
  "0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200";

const formatChannel = (
  res: PromiseType<PaymentChannelInstance["channels"]>
) => ({
  deposit: res[0],
  closure_time: res[1],
  status: res[2]
});

contract("PaymentChannel", ([sender, recipient]) => {
  let hoprToken: HoprTokenInstance;
  let paymentChannel: PaymentChannelInstance;

  const depositAmount = web3.utils.toWei("1", "ether");
  let totalSupply: string;

  before(async () => {
    hoprToken = await HoprToken.new();
    totalSupply = await hoprToken.totalSupply().then(res => res.toString());

    paymentChannel = await PaymentChannel.new(
      hoprToken.address,
      time.duration.days(2)
    );

    await hoprToken.approve(paymentChannel.address, totalSupply);
  });

  it("should have created channel correctly", async () => {
    const receipt = await paymentChannel.createChannel(
      sender,
      sender,
      recipient,
      depositAmount
    );

    expectEvent(receipt, "OpenedChannel", {
      funder: sender,
      sender,
      recipient,
      deposit: depositAmount
    });

    const channel = await paymentChannel
      .channels(sender, recipient)
      .then(formatChannel);

    expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(
      true,
      "wrong deposit"
    );
    expect(channel.closure_time.isZero()).to.be.equal(
      true,
      "wrong closure_time"
    );
    expect(channel.status.eq(new BN(1))).to.be.equal(true, "wrong status");
  });

  it("payment 'signer' should be 'sender'", async () => {
    const signature = signPayment(
      web3,
      senderPrivKey,
      paymentChannel.address,
      web3.utils.toWei("1", "ether").toString()
    );

    const signer = recoverSigner(web3, signature.message, signature.signature);

    expect(signer).to.be.eq(sender, "wrong signer");
  });

  it("should send 0.5 HOPR to 'recipient' and 0.5 HOPR to 'sender'", async () => {
    const amount = web3.utils.toWei("0.5", "ether");

    const payment = signPayment(
      web3,
      senderPrivKey,
      paymentChannel.address,
      amount
    );

    const receipt = await paymentChannel.closeChannel(
      sender,
      amount,
      payment.signature,
      {
        from: recipient
      }
    );

    expectEvent(receipt, "ClosedChannel", {
      sender,
      recipient,
      senderAmount: web3.utils.toWei("0.5", "ether").toString(),
      recipientAmount: web3.utils.toWei("0.5", "ether").toString()
    });

    const channel = await paymentChannel
      .channels(sender, recipient)
      .then(formatChannel);

    expect(channel.status.isZero()).to.be.equal(true, "wrong status");

    const senderBalance = await hoprToken.balanceOf(sender);
    const recipientBalance = await hoprToken.balanceOf(recipient);
    const paymentChannelBalance = await hoprToken.balanceOf(
      paymentChannel.address
    );

    const expectedSenderBalance = new BN(totalSupply).sub(
      new BN(web3.utils.toWei("0.5", "ether"))
    );
    const expectedRecipientBalance = new BN(web3.utils.toWei("0.5", "ether"));

    expect(senderBalance.eq(expectedSenderBalance)).to.be.equal(
      true,
      "wrong senderBalance"
    );
    expect(recipientBalance.eq(expectedRecipientBalance)).to.be.equal(
      true,
      "wrong recipientBalance"
    );
    expect(paymentChannelBalance.isZero()).to.be.equal(
      true,
      "wrong paymentChannelBalance"
    );
  });

  it("should send 0.5 HOPR to 'recipient' and 0.5 HOPR to 'sender' by closure", async () => {
    await paymentChannel.createChannel(
      sender,
      sender,
      recipient,
      depositAmount
    );

    const receipt = await paymentChannel.initiateChannelClosure(recipient);

    expectEvent(receipt, "InitiatedChannelClosure", {
      sender,
      recipient
    });

    await time.increase(time.duration.days(3));
    await paymentChannel.claimChannelClosure(
      recipient,
      web3.utils.toWei("0.5", "ether")
    );

    const channel = await paymentChannel
      .channels(sender, recipient)
      .then(formatChannel);

    expect(channel.status.isZero()).to.be.equal(true, "wrong status");

    const senderBalance = await hoprToken.balanceOf(sender);
    const recipientBalance = await hoprToken.balanceOf(recipient);
    const paymentChannelBalance = await hoprToken.balanceOf(
      paymentChannel.address
    );

    const expectedSenderBalance = new BN(totalSupply).sub(
      new BN(web3.utils.toWei("1", "ether"))
    );
    const expectedRecipientBalance = new BN(web3.utils.toWei("1", "ether"));

    expect(senderBalance.eq(expectedSenderBalance)).to.be.equal(
      true,
      "wrong senderBalance"
    );
    expect(recipientBalance.eq(expectedRecipientBalance)).to.be.equal(
      true,
      "wrong recipientBalance"
    );
    expect(paymentChannelBalance.isZero()).to.be.equal(
      true,
      "wrong paymentChannelBalance"
    );
  });
});
