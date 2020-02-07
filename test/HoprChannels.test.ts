import BN = require("bn.js");
import {
  HoprChannelsContract,
  HoprChannelsInstance,
  HoprTokenContract,
  HoprTokenInstance
} from "../types/truffle-contracts";
import {
  recoverSigner,
  keccak256,
  Ticket,
  getChannelId,
  getParties,
  isPartyA
} from "./utils";
import { PromiseType } from "../types/typescript";
import { time, expectEvent, expectRevert } from "@openzeppelin/test-helpers";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");
const HoprChannels: HoprChannelsContract = artifacts.require("HoprChannels");

// taken from "scripts/test.sh"
const senderPrivKey =
  "0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200";

const formatAccount = (res: PromiseType<HoprChannelsInstance["accounts"]>) => ({
  hashedSecret: res[0],
  counter: res[1]
});

const formatChannel = (res: PromiseType<HoprChannelsInstance["channels"]>) => ({
  deposit: res[0],
  party_a_balance: res[1],
  closureTime: res[2],
  isOpen: res[3]
});

const PrintGasUsed = (name: string) => (
  response: Truffle.TransactionResponse
) => {
  console.log(`gas used in '${name}'`, response.receipt.gasUsed);
  return response;
};

contract("HoprChannels", function([sender, recipient, randomUser]) {
  const depositAmount = web3.utils.toWei("1", "ether");
  let hoprToken: HoprTokenInstance;
  let hoprChannels: HoprChannelsInstance;
  let totalSupply: string;

  const reset = async () => {
    hoprToken = await HoprToken.new();
    // mint supply
    await hoprToken.mint(sender, web3.utils.toWei("100000000", "ether"));
    totalSupply = await hoprToken.totalSupply().then(res => res.toString());

    hoprChannels = await HoprChannels.new(
      hoprToken.address,
      time.duration.days(2)
    );
  };

  // reset contracts for every test
  // describe("unit tests", function() {
  //   beforeEach(async function() {
  //     await reset();
  //   });

  //   it("should have set hashedSecret correctly", async function() {
  //     const secretHash = keccak256({
  //       type: "string",
  //       value: "recipient secret"
  //     });

  //     const response = await hoprChannels.setHashedSecret(secretHash, {
  //       from: recipient
  //     });

  //     expectEvent(response, "SecretHashSet", {
  //       account: recipient,
  //       secretHash
  //     });
  //   });

  //   it("should have created channel correctly", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     const response = await hoprChannels.createChannel(
  //       sender,
  //       sender,
  //       recipient,
  //       depositAmount
  //     );

  //     expectEvent(response, "OpenedChannel", {
  //       funder: sender,
  //       sender,
  //       recipient,
  //       deposit: depositAmount
  //     });

  //     const channel = await hoprChannels
  //       .channels(sender, recipient)
  //       .then(formatChannel);

  //     expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(
  //       true,
  //       "wrong deposit"
  //     );
  //     expect(channel.closureTime.isZero()).to.be.equal(
  //       true,
  //       "wrong closureTime"
  //     );
  //     expect(channel.isOpen).to.be.equal(true, "wrong isOpen");
  //   });

  //   it("payment 'signer' should be 'sender'", async function() {
  //     const ticket = Ticket({
  //       web3,
  //       sender,
  //       recipient,
  //       senderPrivKey,
  //       porSecretA: keccak256({
  //         type: "bytes32",
  //         value: keccak256({ type: "string", value: "por secret a" })
  //       }),
  //       porSecretB: keccak256({
  //         type: "bytes32",
  //         value: keccak256({ type: "string", value: "por secret b" })
  //       }),
  //       recipientSecret: keccak256({
  //         type: "bytes32",
  //         value: keccak256({ type: "string", value: "recipient secret" })
  //       }),
  //       amount: web3.utils.toWei("0.2", "ether"),
  //       counter: 1,
  //       winProbPercent: "100"
  //     });

  //     const signer = recoverSigner(web3, ticket.hashedTicket, ticket.signature);
  //     expect(signer).to.be.eq(sender, "wrong signer");
  //   });

  //   it("should fail when creating an open channel a second time", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       sender,
  //       sender,
  //       recipient,
  //       depositAmount
  //     );

  //     await expectRevert(
  //       hoprChannels.createChannel(sender, sender, recipient, depositAmount),
  //       "channel is already open"
  //     );
  //   });

  //   it("should fail when 'sender' is calling 'closeChannel'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       sender,
  //       sender,
  //       recipient,
  //       depositAmount
  //     );

  //     await expectRevert(
  //       hoprChannels.closeChannel(sender, {
  //         from: sender
  //       }),
  //       "channel must be 'open' or 'pending for closure'"
  //     );
  //   });

  //   it("should fail when 'claimChannelClosure' before closureTime", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       sender,
  //       sender,
  //       recipient,
  //       depositAmount
  //     );

  //     await hoprChannels.initiateChannelClosure(recipient, {
  //       from: sender
  //     });

  //     await expectRevert(
  //       hoprChannels.claimChannelClosure(recipient, {
  //         from: sender
  //       }),
  //       "'closureTime' has not passed"
  //     );
  //   });

  //   it("should fail when calling 'initiateChannelClosure' from 'randomUser'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       sender,
  //       sender,
  //       recipient,
  //       depositAmount
  //     );

  //     await expectRevert(
  //       hoprChannels.initiateChannelClosure(recipient, {
  //         from: randomUser
  //       }),
  //       "channel is not open"
  //     );
  //   });

  //   it("should fail when calling 'claimChannelClosure' from 'randomUser'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       sender,
  //       sender,
  //       recipient,
  //       depositAmount
  //     );

  //     await hoprChannels.initiateChannelClosure(recipient, {
  //       from: sender
  //     });

  //     await expectRevert(
  //       hoprChannels.claimChannelClosure(recipient, {
  //         from: randomUser
  //       }),
  //       "channel is not pending for closure"
  //     );
  //   });

  //   it("should fail 'createChannel' when token balance too low'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);
  //     await hoprToken.burn(totalSupply, {
  //       from: sender
  //     });

  //     await expectRevert(
  //       hoprChannels.createChannel(sender, sender, recipient, depositAmount, {
  //         from: sender
  //       }),
  //       "SafeERC20: low-level call failed"
  //     );
  //   });
  // });

  // reset contracts once
  describe("integration tests", function() {
    before(async function() {
      await reset();
      await hoprToken.approve(hoprChannels.address, totalSupply);
    });

    it("should send 0.2 HOPR to 'recipient' and 0.8 HOPR to 'sender'", async function() {
      const ticket = Ticket({
        web3,
        sender,
        recipient,
        senderPrivKey,
        porSecretA: keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "por secret a" })
        }),
        porSecretB: keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "por secret b" })
        }),
        recipientSecret: keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "recipient secret" })
        }),
        amount: web3.utils.toWei("0.2", "ether"),
        counter: 1,
        winProbPercent: "100"
      });

      const { party_a, party_b } = getParties(sender, recipient);

      await hoprChannels.setHashedSecret(ticket.hashedRecipientSecret, {
        from: recipient
      });

      await hoprChannels
        .fundChannel(sender, recipient, depositAmount)
        .then(PrintGasUsed("createChannel first time"));

      await hoprChannels.openChannel(recipient, {
        from: sender
      });

      // TODO: update to redeem ticket and close channel
      await hoprChannels
        .redeemTicket(
          ticket.recipientSecret,
          ticket.porSecretA,
          ticket.porSecretB,
          ticket.amount,
          ticket.winProb,
          ticket.r,
          ticket.s,
          ticket.v,
          { from: ticket.recipient }
        )
        .then(PrintGasUsed("reedem ticket and close channel"));

      const response = await hoprChannels
        .initiateChannelClosure(recipient)
        .then(PrintGasUsed("initiateChannelClosure"));

      expectEvent(response, "InitiatedChannelClosure", {
        initiator: sender,
        counter_party: recipient
      });

      await time.increase(time.duration.days(3));
      const response2 = await hoprChannels
        .claimChannelClosure(recipient)
        .then(PrintGasUsed("claimChannelClosure"));

      expectEvent(response2, "ClosedChannel", {
        closer: sender,
        counter_party: recipient,
        party_a_amount: isPartyA(sender, recipient)
          ? web3.utils.toWei("0.8", "ether").toString()
          : web3.utils.toWei("0.2", "ether").toString(),
        party_b_amount: isPartyA(recipient, sender)
          ? web3.utils.toWei("0.8", "ether").toString()
          : web3.utils.toWei("0.2", "ether").toString()
      });

      const recipientAccount = await hoprChannels
        .accounts(recipient)
        .then(formatAccount);

      expect(recipientAccount.hashedSecret).to.be.equal(
        ticket.recipientSecret,
        "wrong hashedSecret"
      );

      expect(recipientAccount.counter.eq(new BN(1))).to.be.equal(
        true,
        "wrong counter"
      );

      const channel = await hoprChannels
        .channels(getChannelId(party_a, party_b))
        .then(formatChannel);

      expect(channel.isOpen).to.be.equal(false, "wrong isOpen");

      const senderBalance = await hoprToken.balanceOf(sender);
      const recipientBalance = await hoprToken.balanceOf(recipient);
      const HoprChannelsBalance = await hoprToken.balanceOf(
        hoprChannels.address
      );

      const expectedSenderBalance = new BN(totalSupply).sub(
        new BN(web3.utils.toWei("0.2", "ether"))
      );
      const expectedRecipientBalance = new BN(web3.utils.toWei("0.2", "ether"));

      expect(senderBalance.eq(expectedSenderBalance)).to.be.equal(
        true,
        "wrong senderBalance"
      );
      expect(recipientBalance.eq(expectedRecipientBalance)).to.be.equal(
        true,
        "wrong recipientBalance"
      );
      expect(HoprChannelsBalance.isZero()).to.be.equal(
        true,
        "wrong HoprChannelsBalance"
      );
    });

    it("should send 0.8 HOPR to 'recipient' and 0.2 HOPR to 'sender'", async function() {
      const ticket = Ticket({
        web3,
        sender,
        recipient,
        senderPrivKey,
        porSecretA: keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "por secret a" })
        }),
        porSecretB: keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "por secret b" })
        }),
        recipientSecret: keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "recipient secret" })
        }),
        amount: web3.utils.toWei("0.8", "ether"),
        counter: 2,
        winProbPercent: "100"
      });

      const { party_a, party_b } = getParties(sender, recipient);

      await hoprChannels.setHashedSecret(ticket.hashedRecipientSecret, {
        from: recipient
      });

      await hoprChannels
        .fundChannel(sender, recipient, depositAmount)
        .then(PrintGasUsed("createChannel a second time"));

      await hoprChannels.openChannel(recipient, {
        from: sender
      });

      await hoprChannels
        .redeemTicket(
          ticket.recipientSecret,
          ticket.porSecretA,
          ticket.porSecretB,
          ticket.amount,
          ticket.winProb,
          ticket.r,
          ticket.s,
          ticket.v,
          { from: ticket.recipient }
        )
        .then(PrintGasUsed("reedem ticket"));

      const response = await hoprChannels
        .initiateChannelClosure(recipient)
        .then(PrintGasUsed("initiateChannelClosure"));

      expectEvent(response, "InitiatedChannelClosure", {
        initiator: sender,
        counter_party: recipient
      });

      await time.increase(time.duration.days(3));
      const response2 = await hoprChannels
        .claimChannelClosure(recipient)
        .then(PrintGasUsed("claimChannelClosure"));

      expectEvent(response2, "ClosedChannel", {
        closer: sender,
        counter_party: recipient,
        party_a_amount: isPartyA(sender, recipient)
          ? web3.utils.toWei("0.2", "ether").toString()
          : web3.utils.toWei("0.8", "ether").toString(),
        party_b_amount: isPartyA(recipient, sender)
          ? web3.utils.toWei("0.2", "ether").toString()
          : web3.utils.toWei("0.8", "ether").toString()
      });

      const recipientAccount = await hoprChannels
        .accounts(recipient)
        .then(formatAccount);

      expect(recipientAccount.hashedSecret).to.be.equal(
        ticket.recipientSecret,
        "wrong hashedSecret"
      );

      expect(recipientAccount.counter.eq(new BN(2))).to.be.equal(
        true,
        "wrong counter"
      );

      const channel = await hoprChannels
        .channels(getChannelId(party_a, party_b))
        .then(formatChannel);

      expect(channel.isOpen).to.be.equal(false, "wrong isOpen");

      const senderBalance = await hoprToken.balanceOf(sender);
      const recipientBalance = await hoprToken.balanceOf(recipient);
      const HoprChannelsBalance = await hoprToken.balanceOf(
        hoprChannels.address
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
      expect(HoprChannelsBalance.isZero()).to.be.equal(
        true,
        "wrong HoprChannelsBalance"
      );
    });
  });
});
