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
import { beforeEach } from "mocha";

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
  sender: res[0],
  recipient: res[1],
  deposit: res[2],
  expiration_time: res[3],
  closure_amount: res[4],
  closure_time: res[5],
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

    paymentChannel = await PaymentChannel.new(
      hoprToken.address,
      time.duration.weeks(1),
      time.duration.days(2)
    );

    await hoprToken.approve(paymentChannel.address, totalSupply);
  });

  context("the recipient is closing the channel", () => {
    it("should have created channel correctly", async () => {
      const receipt = await paymentChannel.createChannel(
        sender,
        sender,
        recipient,
        depositAmount
      );

      expectEvent(receipt, "OpenedChannel", {
        id: "1",
        funder: sender,
        sender,
        recipient,
        deposit: depositAmount
      });

      const channel = await paymentChannel.channels("1").then(formatChannel);
      const numberOfChannels = await paymentChannel.numberOfChannels();

      expect(channel.sender).to.be.equal(sender);
      expect(channel.recipient).to.be.equal(recipient);
      expect(channel.status.toString()).to.be.equal("1");
      expect(numberOfChannels.toString()).to.be.equal("1");
    });

    it("payment 'signer' should be 'sender'", async () => {
      const signature = signPayment(
        web3,
        senderPrivKey,
        paymentChannel.address,
        web3.utils.toWei("1", "ether").toString()
      );

      const signer = recoverSigner(
        web3,
        signature.message,
        signature.signature
      );

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

      const receipt = await paymentChannel.closeChannel(
        "1",
        amount,
        payment.signature,
        {
          from: recipient
        }
      );

      expectEvent(receipt, "ClosedChannel", {
        id: "1",
        senderAmount: web3.utils.toWei("0.5", "ether").toString(),
        recipientAmount: web3.utils.toWei("0.5", "ether").toString()
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
        sender,
        sender,
        recipient,
        depositAmount
      );

      await time.increase(time.duration.weeks(2));
      await paymentChannel.claimChannelExpiration("2");

      const channel = await paymentChannel.channels("2").then(formatChannel);
      expect(channel.status.toString()).to.be.equal("3");
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

  context("the sender is closing the channel", () => {
    it("should send 0.5 HOPR to 'recipient' and 0.5 HOPR to 'sender'", async () => {
      await paymentChannel.createChannel(
        sender,
        sender,
        recipient,
        depositAmount
      );

      const receipt = await paymentChannel.initiateChannelClosure(
        "3",
        web3.utils.toWei("0.5", "ether")
      );

      expectEvent(receipt, "InitiatedChannelClosure", {
        id: "3",
        closure_amount: web3.utils.toWei("0.5", "ether").toString()
      });

      await time.increase(time.duration.days(3));
      await paymentChannel.claimChannelClosure("3");

      const channel = await paymentChannel.channels("3").then(formatChannel);
      expect(channel.status.toString()).to.be.equal("3");
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
          .sub(new BN(web3.utils.toWei("1", "ether")))
          .toString()
      );
      expect(recipientBalance).to.be.equal(web3.utils.toWei("1", "ether"));
      expect(paymentChannelBalance).to.be.equal("0");
    });

    it("should send 0.5 HOPR to 'recipient' and 0.5 HOPR to 'sender'", async () => {
      await paymentChannel.createChannel(
        sender,
        sender,
        recipient,
        depositAmount
      );

      await paymentChannel.initiateChannelClosure(
        "4",
        web3.utils.toWei("0.5", "ether")
      );

      await time.increase(time.duration.weeks(2));
      await paymentChannel.claimChannelClosure("4");

      const channel = await paymentChannel.channels("4").then(formatChannel);
      expect(channel.status.toString()).to.be.equal("3");
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
          .sub(new BN(web3.utils.toWei("1.5", "ether")))
          .toString()
      );
      expect(recipientBalance).to.be.equal(web3.utils.toWei("1.5", "ether"));
      expect(paymentChannelBalance).to.be.equal("0");
    });
  });
});
