import BN = require("bn.js");
import {
  PaymentChannelContract,
  PaymentChannelInstance,
  HoprTokenContract,
  HoprTokenInstance
} from "../../../types/truffle-contracts";
import { signPayment, recoverSigner } from "./utils";
import { PromiseType } from "../../../types/typescript";
import { time } from "@openzeppelin/test-helpers";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");

const PaymentChannel: PaymentChannelContract = artifacts.require(
  "PaymentChannel"
);

const senderPrivKey =
  "0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200";

const responseToChannel = (
  res: PromiseType<PaymentChannelInstance["channels"]>
) => ({
  id: res[0],
  sender: res[1],
  recipient: res[2],
  token: res[3],
  deposit: res[4],
  expiration: res[5],
  status: res[6]
});

contract("PaymentChannel", ([sender, recipient]) => {
  let hoprToken: HoprTokenInstance;
  let paymentChannel: PaymentChannelInstance;

  const depositAmount = web3.utils.toWei("1", "ether");
  let totalSupply: string;

  before(async () => {
    hoprToken = await HoprToken.new();
    totalSupply = await hoprToken.totalSupply().then(res => res.toString());

    paymentChannel = await PaymentChannel.new();

    await hoprToken.approve(paymentChannel.address, totalSupply);
  });

  it("should have created channel correctly", async () => {
    await paymentChannel.createChannel(
      recipient,
      hoprToken.address,
      depositAmount,
      time.duration.weeks(1)
    );

    const channel = await paymentChannel.channels("1").then(responseToChannel);
    const numberOfChannels = await paymentChannel.numberOfChannels();

    expect(channel.id.toString()).to.be.equal("1");
    expect(channel.sender).to.be.equal(sender);
    expect(channel.recipient).to.be.equal(recipient);
    expect(channel.token).to.be.equal(hoprToken.address);
    expect(channel.status.toString()).to.be.equal("0");
    expect(numberOfChannels.toString()).to.be.equal("1");
  });

  it("payment 'signer' should be 'sender'", async () => {
    const signature = signPayment(
      web3,
      senderPrivKey,
      paymentChannel.address,
      web3.utils.toWei("1", "ether").toString()
    );

    const signer = recoverSigner(web3, signature.message, signature.signature);

    expect(signer).to.be.equal(sender);
  });

  it("should send 0.5 HOPR to 'recipient' and 0.5 HOPR to 'sender'", async () => {
    const amount = web3.utils.toWei("0.5", "ether");

    const payment = signPayment(
      web3,
      senderPrivKey,
      paymentChannel.address,
      amount
    );

    await paymentChannel.closeChannel("1", amount, payment.signature, {
      from: recipient
    });

    const senderBalance = await hoprToken
      .balanceOf(sender)
      .then(res => res.toString());

    const recipientBalance = await hoprToken
      .balanceOf(recipient)
      .then(res => res.toString());

    const paymentChannelBalance = await hoprToken
      .balanceOf(paymentChannel.address)
      .then(res => res.toString());

    expect(senderBalance).to.be.equal(
      new BN(totalSupply)
        .sub(new BN(web3.utils.toWei("0.5", "ether")))
        .toString()
    );
    expect(recipientBalance).to.be.equal(web3.utils.toWei("0.5", "ether"));
    expect(paymentChannelBalance).to.be.equal("0");
  });

  it("should send 1 HOPR to 'sender' on timeout", async () => {
    await paymentChannel.createChannel(
      recipient,
      hoprToken.address,
      depositAmount,
      time.duration.weeks(1)
    );

    await time.increase(time.duration.weeks(2));
    await paymentChannel.claimChannelTimeout("2");

    const channel = await paymentChannel.channels("2").then(responseToChannel);
    expect(channel.status.toString()).to.be.equal("1");
    expect(channel.deposit.toString()).to.be.equal("0");

    const senderBalance = await hoprToken
      .balanceOf(sender)
      .then(res => res.toString());

    const recipientBalance = await hoprToken
      .balanceOf(recipient)
      .then(res => res.toString());

    const paymentChannelBalance = await hoprToken
      .balanceOf(paymentChannel.address)
      .then(res => res.toString());

    expect(senderBalance).to.be.equal(
      new BN(totalSupply)
        .sub(new BN(web3.utils.toWei("0.5", "ether")))
        .toString()
    );
    expect(recipientBalance).to.be.equal(web3.utils.toWei("0.5", "ether"));
    expect(paymentChannelBalance).to.be.equal("0");
  });
});
