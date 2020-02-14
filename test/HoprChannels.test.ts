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
  isPartyA,
  Fund
} from "./utils";
import { PromiseType } from "../types/typescript";
import { time, expectEvent, expectRevert } from "@openzeppelin/test-helpers";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");
const HoprChannels: HoprChannelsContract = artifacts.require("HoprChannels");

const formatAccount = (res: PromiseType<HoprChannelsInstance["accounts"]>) => ({
  hashedSecret: res[0],
  counter: res[1]
});

const formatChannel = (res: PromiseType<HoprChannelsInstance["channels"]>) => ({
  deposit: res[0],
  party_a_balance: res[1],
  closureTime: res[2],
  state_counter: res[3]
});

const PrintGasUsed = (name: string) => (
  response: Truffle.TransactionResponse
) => {
  console.log(`gas used in '${name}'`, response.receipt.gasUsed);
  return response;
};

contract("HoprChannels", function([accountA, accountB, randomUser]) {
  const { partyA, partyB } = getParties(accountA, accountB);

  // taken from "scripts/test.sh"
  const partyAPrivKey =
    "0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201";
  const partyBPrivKey =
    "0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200";

  const depositAmount = web3.utils.toWei("1", "ether");
  let hoprToken: HoprTokenInstance;
  let hoprChannels: HoprChannelsInstance;
  let totalSupply: string;

  const reset = async () => {
    hoprToken = await HoprToken.new();
    // mint supply
    await hoprToken.mint(partyA, web3.utils.toWei("100", "ether"));
    await hoprToken.mint(partyB, web3.utils.toWei("100", "ether"));
    totalSupply = await hoprToken.totalSupply().then(res => res.toString());

    hoprChannels = await HoprChannels.new(
      hoprToken.address,
      time.duration.days(2)
    );
  };

  // reset contracts once
  describe("integration tests", function() {
    before(async function() {
      await reset();
    });

    context(
      "make payments between 'partyA' and 'partyB' using a fresh channel and 'fundChannel'",
      function() {
        const partyASecret1 = keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "partyA secret 1" })
        });
        const partyASecret2 = keccak256({
          type: "bytes32",
          value: partyASecret1
        });

        const partyBSecret1 = keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "partyB secret 1" })
        });
        const partyBSecret2 = keccak256({
          type: "bytes32",
          value: partyBSecret1
        });

        it("'partyA' should fund 'partyA' with 1 HOPR", async function() {
          await hoprToken.approve(hoprChannels.address, totalSupply, {
            from: partyA
          });

          const result = await hoprChannels.fundChannel(
            partyA,
            partyB,
            depositAmount,
            {
              from: partyA
            }
          );

          expectEvent(result, "FundedChannel", {
            funder: partyA,
            recipient: partyA,
            counter_party: partyB,
            recipient_amount: depositAmount,
            counter_party_amount: new BN(0)
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong deposit"
          );
          expect(channel.party_a_balance.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong party_a_balance"
          );
          expect(channel.state_counter.eq(new BN(1))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should fund 'partyB' with 1 HOPR", async function() {
          await hoprToken.approve(hoprChannels.address, totalSupply, {
            from: partyB
          });

          const result = await hoprChannels.fundChannel(
            partyB,
            partyA,
            depositAmount,
            {
              from: partyB
            }
          );

          expectEvent(result, "FundedChannel", {
            funder: partyB,
            recipient: partyB,
            counter_party: partyA,
            recipient_amount: depositAmount,
            counter_party_amount: new BN(0)
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(
            channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))
          ).to.be.equal(true, "wrong deposit");

          expect(channel.party_a_balance.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong party_a_balance"
          );

          expect(channel.state_counter.eq(new BN(1))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("should set hashed secret for 'partyA'", async function() {
          // make a ticket to generate hashedSecret for 'partyA'
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyASecret2,
            amount: web3.utils.toWei("0.2", "ether"),
            counter: 1,
            winProbPercent: "100"
          });

          await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
            from: partyA
          });

          const partyAAccount = await hoprChannels
            .accounts(partyA)
            .then(formatAccount);

          expect(partyAAccount.hashedSecret).to.be.equal(
            ticket.hashedCounterPartySecret,
            "wrong hashedSecret"
          );

          expect(partyAAccount.counter.eq(new BN(1))).to.be.equal(
            true,
            "wrong counter"
          );
        });

        it("should set hashed secret for 'partyB'", async function() {
          // make a ticket to generate hashedSecret for 'partyB'
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyBSecret2,
            amount: web3.utils.toWei("0.2", "ether"),
            counter: 1,
            winProbPercent: "100"
          });

          await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
            from: partyB
          });

          const partyBAccount = await hoprChannels
            .accounts(partyB)
            .then(formatAccount);

          expect(partyBAccount.hashedSecret).to.be.equal(
            ticket.hashedCounterPartySecret,
            "wrong hashedSecret"
          );

          expect(partyBAccount.counter.eq(new BN(1))).to.be.equal(
            true,
            "wrong counter"
          );
        });

        it("should open channel", async function() {
          const result = await hoprChannels.openChannel(partyB, {
            from: partyA
          });

          expectEvent(result, "OpenedChannel", {
            opener: partyA,
            counter_party: partyB
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.state_counter.eq(new BN(2))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyA' should reedem winning ticket of 0.2 HOPR", async function() {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyBPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyASecret2,
            amount: web3.utils.toWei("0.2", "ether"),
            counter: 1,
            winProbPercent: "100"
          });

          await hoprChannels.redeemTicket(
            ticket.counterPartySecret,
            ticket.porSecretA,
            ticket.porSecretB,
            ticket.amount,
            ticket.winProb,
            ticket.r,
            ticket.s,
            ticket.v,
            { from: partyA }
          );

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(
            channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))
          ).to.be.equal(true, "wrong deposit");

          expect(
            channel.party_a_balance.eq(
              new BN(depositAmount).add(
                new BN(web3.utils.toWei("0.2", "ether"))
              )
            )
          ).to.be.equal(true, "wrong party_a_balance");

          expect(channel.state_counter.eq(new BN(2))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should reedem winning ticket of 1.2 HOPR", async function() {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyBSecret2,
            amount: web3.utils.toWei("1.2", "ether"),
            counter: 1,
            winProbPercent: "100"
          });

          await hoprChannels.redeemTicket(
            ticket.counterPartySecret,
            ticket.porSecretA,
            ticket.porSecretB,
            ticket.amount,
            ticket.winProb,
            ticket.r,
            ticket.s,
            ticket.v,
            { from: partyB }
          );

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(
            channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))
          ).to.be.equal(true, "wrong deposit");

          expect(channel.party_a_balance.eq(new BN(0))).to.be.equal(
            true,
            "wrong party_a_balance"
          );

          expect(channel.state_counter.eq(new BN(2))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should initiate closure", async function() {
          const result = await hoprChannels.initiateChannelClosure(partyA, {
            from: partyB
          });

          expectEvent(result, "InitiatedChannelClosure", {
            initiator: partyB,
            counter_party: partyA
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.state_counter.eq(new BN(3))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyA' should reedem winning ticket of 0.5 HOPR", async function() {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyBPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyASecret1,
            amount: web3.utils.toWei("0.5", "ether"),
            counter: 1,
            winProbPercent: "100"
          });

          await hoprChannels.redeemTicket(
            ticket.counterPartySecret,
            ticket.porSecretA,
            ticket.porSecretB,
            ticket.amount,
            ticket.winProb,
            ticket.r,
            ticket.s,
            ticket.v,
            { from: partyA }
          );

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(
            channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))
          ).to.be.equal(true, "wrong deposit");

          expect(
            channel.party_a_balance.eq(new BN(web3.utils.toWei("0.5", "ether")))
          ).to.be.equal(true, "wrong party_a_balance");

          expect(channel.state_counter.eq(new BN(3))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyA' should close channel", async function() {
          await time.increase(time.duration.days(3));

          const result = await hoprChannels.claimChannelClosure(partyB, {
            from: partyA
          });

          expectEvent(result, "ClosedChannel", {
            closer: partyA,
            counter_party: partyB,
            party_a_amount: web3.utils.toWei("0.5", "ether"),
            party_b_amount: web3.utils.toWei("1.5", "ether")
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(0))).to.be.equal(
            true,
            "wrong deposit"
          );

          expect(channel.party_a_balance.eq(new BN(0))).to.be.equal(
            true,
            "wrong party_a_balance"
          );

          expect(channel.state_counter.eq(new BN(10))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });
      }
    );

    context.skip(
      "make payments between 'partyA' and 'partyB' using a recycled channel and 'fundChannelWithSig'",
      function() {
        const partyASecret1 = keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "partyA secret 2" })
        });
        const partyASecret2 = keccak256({
          type: "bytes32",
          value: partyASecret1
        });

        const partyBSecret1 = keccak256({
          type: "bytes32",
          value: keccak256({ type: "string", value: "partyB secret 2" })
        });
        const partyBSecret2 = keccak256({
          type: "bytes32",
          value: partyBSecret1
        });

        it("'partyA' should fund 'partyA' with 0.2 HOPR", async function() {
          const partyAAmount = web3.utils.toWei("0.2", "ether");

          const not_after = await time.latest().then(now => {
            return now.add(time.duration.days(2)).toString();
          });

          const fund = Fund({
            web3,
            state_counter: "10",
            initiator: partyA,
            deposit: partyAAmount,
            party_a_amount: partyAAmount,
            not_after,
            signerPrivKey: partyBPrivKey
          });

          const result = await hoprChannels.fundChannelWithSig(
            "10",
            partyAAmount,
            partyAAmount,
            not_after,
            fund.r,
            fund.s,
            fund.v,
            {
              from: partyA
            }
          );

          expectEvent(result, "FundedChannel", {
            // funder: partyA,
            recipient: partyA,
            counter_party: partyB,
            recipient_amount: partyAAmount,
            counter_party_amount: new BN(0)
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(partyAAmount))).to.be.equal(
            true,
            "wrong deposit"
          );
          expect(channel.party_a_balance.eq(new BN(partyAAmount))).to.be.equal(
            true,
            "wrong party_a_balance"
          );
          expect(channel.state_counter.eq(new BN(11))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should fund 'partyB' with 0.8 HOPR", async function() {
          const partyBAmount = web3.utils.toWei("0.8", "ether");

          const result = await hoprChannels.fundChannel(
            partyB,
            partyA,
            partyBAmount,
            {
              from: partyB
            }
          );

          expectEvent(result, "FundedChannel", {
            funder: partyB,
            recipient: partyB,
            counter_party: partyA,
            recipient_amount: partyBAmount,
            counter_party_amount: new BN(0)
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(
            channel.deposit.eq(new BN(web3.utils.toWei("1", "ether")))
          ).to.be.equal(true, "wrong deposit");

          expect(
            channel.party_a_balance.eq(new BN(web3.utils.toWei("0.2", "ether")))
          ).to.be.equal(true, "wrong party_a_balance");

          expect(channel.state_counter.eq(new BN(11))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("should set hashed secret for 'partyA'", async function() {
          // make a ticket to generate hashedSecret for 'partyA'
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyASecret2,
            amount: web3.utils.toWei("0.3", "ether"),
            counter: 2,
            winProbPercent: "100"
          });

          await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
            from: partyA
          });

          const partyAAccount = await hoprChannels
            .accounts(partyA)
            .then(formatAccount);

          expect(partyAAccount.hashedSecret).to.be.equal(
            ticket.hashedCounterPartySecret,
            "wrong hashedSecret"
          );

          expect(partyAAccount.counter.eq(new BN(2))).to.be.equal(
            true,
            "wrong counter"
          );
        });

        it("should set hashed secret for 'partyB'", async function() {
          // make a ticket to generate hashedSecret for 'partyB'
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyBSecret2,
            amount: web3.utils.toWei("0.7", "ether"),
            counter: 2,
            winProbPercent: "100"
          });

          await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
            from: partyB
          });

          const partyBAccount = await hoprChannels
            .accounts(partyB)
            .then(formatAccount);

          expect(partyBAccount.hashedSecret).to.be.equal(
            ticket.hashedCounterPartySecret,
            "wrong hashedSecret"
          );

          expect(partyBAccount.counter.eq(new BN(2))).to.be.equal(
            true,
            "wrong counter"
          );
        });

        it("should open channel", async function() {
          const result = await hoprChannels.openChannel(partyB, {
            from: partyA
          });

          expectEvent(result, "OpenedChannel", {
            opener: partyA,
            counter_party: partyB
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.state_counter.eq(new BN(12))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyA' should reedem winning ticket of 0.3 HOPR", async function() {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyBPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyASecret2,
            amount: web3.utils.toWei("0.3", "ether"),
            counter: 2,
            winProbPercent: "100"
          });

          await hoprChannels.redeemTicket(
            ticket.counterPartySecret,
            ticket.porSecretA,
            ticket.porSecretB,
            ticket.amount,
            ticket.winProb,
            ticket.r,
            ticket.s,
            ticket.v,
            { from: partyA }
          );

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong deposit"
          );

          expect(
            channel.party_a_balance.eq(new BN(web3.utils.toWei("0.5", "ether")))
          ).to.be.equal(true, "wrong party_a_balance");

          expect(channel.state_counter.eq(new BN(12))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should reedem winning ticket of 0.5 HOPR", async function() {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyBSecret2,
            amount: web3.utils.toWei("0.5", "ether"),
            counter: 2,
            winProbPercent: "100"
          });

          await hoprChannels.redeemTicket(
            ticket.counterPartySecret,
            ticket.porSecretA,
            ticket.porSecretB,
            ticket.amount,
            ticket.winProb,
            ticket.r,
            ticket.s,
            ticket.v,
            { from: partyB }
          );

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong deposit"
          );

          expect(channel.party_a_balance.eq(new BN(0))).to.be.equal(
            true,
            "wrong party_a_balance"
          );

          expect(channel.state_counter.eq(new BN(12))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should initiate closure", async function() {
          const result = await hoprChannels.initiateChannelClosure(partyA, {
            from: partyB
          });

          expectEvent(result, "InitiatedChannelClosure", {
            initiator: partyB,
            counter_party: partyA
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.state_counter.eq(new BN(13))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyA' should reedem winning ticket of 1 HOPR", async function() {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyBPrivKey,
            porSecretA: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret a" })
            }),
            porSecretB: keccak256({
              type: "bytes32",
              value: keccak256({ type: "string", value: "por secret b" })
            }),
            counterPartySecret: partyASecret1,
            amount: web3.utils.toWei("1", "ether"),
            counter: 2,
            winProbPercent: "100"
          });

          await hoprChannels.redeemTicket(
            ticket.counterPartySecret,
            ticket.porSecretA,
            ticket.porSecretB,
            ticket.amount,
            ticket.winProb,
            ticket.r,
            ticket.s,
            ticket.v,
            { from: partyA }
          );

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong deposit"
          );

          expect(channel.party_a_balance.eq(new BN(depositAmount))).to.be.equal(
            true,
            "wrong party_a_balance"
          );

          expect(channel.state_counter.eq(new BN(13))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });

        it("'partyB' should close channel", async function() {
          await time.increase(time.duration.days(3));

          const result = await hoprChannels.claimChannelClosure(partyA, {
            from: partyB
          });

          expectEvent(result, "ClosedChannel", {
            closer: partyB,
            counter_party: partyA,
            party_a_amount: web3.utils.toWei("1", "ether"),
            party_b_amount: "0"
          });

          const channel = await hoprChannels
            .channels(getChannelId(partyA, partyB))
            .then(formatChannel);

          expect(channel.deposit.eq(new BN(0))).to.be.equal(
            true,
            "wrong deposit"
          );

          expect(channel.party_a_balance.eq(new BN(0))).to.be.equal(
            true,
            "wrong party_a_balance"
          );

          expect(channel.state_counter.eq(new BN(20))).to.be.equal(
            true,
            "wrong state_counter"
          );
        });
      }
    );

    // it("should send 0.2 HOPR to 'partyB' and 0.8 HOPR to 'partyA'", async function() {

    //   const { party_a, party_b } = getParties(partyA, partyB);

    //   await hoprChannels.setHashedSecret(ticket.hashedRecipientSecret, {
    //     from: partyB
    //   });

    //   await hoprChannels
    //     .fundChannel(partyA, partyB, depositAmount)
    //     .then(PrintGasUsed("createChannel first time"));

    //   await hoprChannels.openChannel(partyB, {
    //     from: partyA
    //   });

    //   await hoprChannels
    //     .redeemTicket(
    //       ticket.recipientSecret,
    //       ticket.porSecretA,
    //       ticket.porSecretB,
    //       ticket.amount,
    //       ticket.winProb,
    //       ticket.r,
    //       ticket.s,
    //       ticket.v,
    //       { from: ticket.partyB }
    //     )
    //     .then(PrintGasUsed("reedem ticket and close channel"));

    //   const response = await hoprChannels
    //     .initiateChannelClosure(partyB)
    //     .then(PrintGasUsed("initiateChannelClosure"));

    //   expectEvent(response, "InitiatedChannelClosure", {
    //     initiator: partyA,
    //     counter_party: partyB
    //   });

    //   await time.increase(time.duration.days(3));
    //   const response2 = await hoprChannels
    //     .claimChannelClosure(partyB)
    //     .then(PrintGasUsed("claimChannelClosure"));

    //   expectEvent(response2, "ClosedChannel", {
    //     closer: partyA,
    //     counter_party: partyB,
    //     party_a_amount: isPartyA(partyA, partyB)
    //       ? web3.utils.toWei("0.8", "ether").toString()
    //       : web3.utils.toWei("0.2", "ether").toString(),
    //     party_b_amount: isPartyA(partyB, partyA)
    //       ? web3.utils.toWei("0.8", "ether").toString()
    //       : web3.utils.toWei("0.2", "ether").toString()
    //   });

    //   const recipientAccount = await hoprChannels
    //     .accounts(partyB)
    //     .then(formatAccount);

    //   expect(recipientAccount.hashedSecret).to.be.equal(
    //     ticket.recipientSecret,
    //     "wrong hashedSecret"
    //   );

    //   expect(recipientAccount.counter.eq(new BN(1))).to.be.equal(
    //     true,
    //     "wrong counter"
    //   );

    //   const channel = await hoprChannels
    //     .channels(getChannelId(party_a, party_b))
    //     .then(formatChannel);

    //   expect(channel.state_counter.eq(new BN(4))).to.be.equal(
    //     true,
    //     "wrong state_counter"
    //   );

    //   const senderBalance = await hoprToken.balanceOf(partyA);
    //   const recipientBalance = await hoprToken.balanceOf(partyB);
    //   const HoprChannelsBalance = await hoprToken.balanceOf(
    //     hoprChannels.address
    //   );

    //   const expectedSenderBalance = new BN(totalSupply).sub(
    //     new BN(web3.utils.toWei("0.2", "ether"))
    //   );
    //   const expectedRecipientBalance = new BN(web3.utils.toWei("0.2", "ether"));

    //   expect(senderBalance.eq(expectedSenderBalance)).to.be.equal(
    //     true,
    //     "wrong senderBalance"
    //   );
    //   expect(recipientBalance.eq(expectedRecipientBalance)).to.be.equal(
    //     true,
    //     "wrong recipientBalance"
    //   );
    //   expect(HoprChannelsBalance.isZero()).to.be.equal(
    //     true,
    //     "wrong HoprChannelsBalance"
    //   );
    // });

    // it("should send 0.8 HOPR to 'partyB' and 0.2 HOPR to 'partyA' using sig", async function() {
    //   const ticket = Ticket({
    //     web3,
    //     partyA,
    //     partyB,
    //     senderPrivKey,
    //     porSecretA: keccak256({
    //       type: "bytes32",
    //       value: keccak256({ type: "string", value: "por secret a" })
    //     }),
    //     porSecretB: keccak256({
    //       type: "bytes32",
    //       value: keccak256({ type: "string", value: "por secret b" })
    //     }),
    //     recipientSecret: keccak256({
    //       type: "bytes32",
    //       value: keccak256({ type: "string", value: "partyB secret" })
    //     }),
    //     amount: web3.utils.toWei("0.8", "ether"),
    //     counter: 2,
    //     winProbPercent: "100"
    //   });

    //   const { party_a, party_b } = getParties(partyA, partyB);

    //   await hoprChannels.setHashedSecret(ticket.hashedRecipientSecret, {
    //     from: partyB
    //   });

    //   await hoprChannels
    //     .fundChannel(partyA, partyB, depositAmount)
    //     .then(PrintGasUsed("createChannel a second time"));

    //   await hoprChannels.openChannel(partyB, {
    //     from: partyA
    //   });

    //   await hoprChannels
    //     .redeemTicket(
    //       ticket.recipientSecret,
    //       ticket.porSecretA,
    //       ticket.porSecretB,
    //       ticket.amount,
    //       ticket.winProb,
    //       ticket.r,
    //       ticket.s,
    //       ticket.v,
    //       { from: ticket.partyB }
    //     )
    //     .then(PrintGasUsed("reedem ticket"));

    //   const response = await hoprChannels
    //     .initiateChannelClosure(partyB)
    //     .then(PrintGasUsed("initiateChannelClosure"));

    //   expectEvent(response, "InitiatedChannelClosure", {
    //     initiator: partyA,
    //     counter_party: partyB
    //   });

    //   await time.increase(time.duration.days(3));
    //   const response2 = await hoprChannels
    //     .claimChannelClosure(partyB)
    //     .then(PrintGasUsed("claimChannelClosure"));

    //   expectEvent(response2, "ClosedChannel", {
    //     closer: partyA,
    //     counter_party: partyB,
    //     party_a_amount: isPartyA(partyA, partyB)
    //       ? web3.utils.toWei("0.2", "ether").toString()
    //       : web3.utils.toWei("0.8", "ether").toString(),
    //     party_b_amount: isPartyA(partyB, partyA)
    //       ? web3.utils.toWei("0.2", "ether").toString()
    //       : web3.utils.toWei("0.8", "ether").toString()
    //   });

    //   const recipientAccount = await hoprChannels
    //     .accounts(partyB)
    //     .then(formatAccount);

    //   expect(recipientAccount.hashedSecret).to.be.equal(
    //     ticket.recipientSecret,
    //     "wrong hashedSecret"
    //   );

    //   expect(recipientAccount.counter.eq(new BN(2))).to.be.equal(
    //     true,
    //     "wrong counter"
    //   );

    //   const channel = await hoprChannels
    //     .channels(getChannelId(party_a, party_b))
    //     .then(formatChannel);

    //   expect(channel.state_counter.eq(new BN(8))).to.be.equal(
    //     true,
    //     "wrong state_counter"
    //   );

    //   const senderBalance = await hoprToken.balanceOf(partyA);
    //   const recipientBalance = await hoprToken.balanceOf(partyB);
    //   const HoprChannelsBalance = await hoprToken.balanceOf(
    //     hoprChannels.address
    //   );

    //   const expectedSenderBalance = new BN(totalSupply).sub(
    //     new BN(web3.utils.toWei("1", "ether"))
    //   );
    //   const expectedRecipientBalance = new BN(web3.utils.toWei("1", "ether"));

    //   expect(senderBalance.eq(expectedSenderBalance)).to.be.equal(
    //     true,
    //     "wrong senderBalance"
    //   );
    //   expect(recipientBalance.eq(expectedRecipientBalance)).to.be.equal(
    //     true,
    //     "wrong recipientBalance"
    //   );
    //   expect(HoprChannelsBalance.isZero()).to.be.equal(
    //     true,
    //     "wrong HoprChannelsBalance"
    //   );
    // });
  });

  // reset contracts for every test
  // describe("unit tests", function() {
  //   beforeEach(async function() {
  //     await reset();
  //   });

  //   it("should have set hashedSecret correctly", async function() {
  //     const secretHash = keccak256({
  //       type: "string",
  //       value: "partyB secret"
  //     });

  //     const response = await hoprChannels.setHashedSecret(secretHash, {
  //       from: partyB
  //     });

  //     expectEvent(response, "SecretHashSet", {
  //       account: partyB,
  //       secretHash
  //     });
  //   });

  //   it("should have created channel correctly", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     const response = await hoprChannels.createChannel(
  //       partyA,
  //       partyA,
  //       partyB,
  //       depositAmount
  //     );

  //     expectEvent(response, "OpenedChannel", {
  //       funder: partyA,
  //       partyA,
  //       partyB,
  //       deposit: depositAmount
  //     });

  //     const channel = await hoprChannels
  //       .channels(partyA, partyB)
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

  //   it("payment 'signer' should be 'partyA'", async function() {
  //     const ticket = Ticket({
  //       web3,
  //       partyA,
  //       partyB,
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
  //         value: keccak256({ type: "string", value: "partyB secret" })
  //       }),
  //       amount: web3.utils.toWei("0.2", "ether"),
  //       counter: 1,
  //       winProbPercent: "100"
  //     });

  //     const signer = recoverSigner(web3, ticket.hashedTicket, ticket.signature);
  //     expect(signer).to.be.eq(partyA, "wrong signer");
  //   });

  //   it("should fail when creating an open channel a second time", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       partyA,
  //       partyA,
  //       partyB,
  //       depositAmount
  //     );

  //     await expectRevert(
  //       hoprChannels.createChannel(partyA, partyA, partyB, depositAmount),
  //       "channel is already open"
  //     );
  //   });

  //   it("should fail when 'partyA' is calling 'closeChannel'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       partyA,
  //       partyA,
  //       partyB,
  //       depositAmount
  //     );

  //     await expectRevert(
  //       hoprChannels.closeChannel(partyA, {
  //         from: partyA
  //       }),
  //       "channel must be 'open' or 'pending for closure'"
  //     );
  //   });

  //   it("should fail when 'claimChannelClosure' before closureTime", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       partyA,
  //       partyA,
  //       partyB,
  //       depositAmount
  //     );

  //     await hoprChannels.initiateChannelClosure(partyB, {
  //       from: partyA
  //     });

  //     await expectRevert(
  //       hoprChannels.claimChannelClosure(partyB, {
  //         from: partyA
  //       }),
  //       "'closureTime' has not passed"
  //     );
  //   });

  //   it("should fail when calling 'initiateChannelClosure' from 'randomUser'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       partyA,
  //       partyA,
  //       partyB,
  //       depositAmount
  //     );

  //     await expectRevert(
  //       hoprChannels.initiateChannelClosure(partyB, {
  //         from: randomUser
  //       }),
  //       "channel is not open"
  //     );
  //   });

  //   it("should fail when calling 'claimChannelClosure' from 'randomUser'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);

  //     await hoprChannels.createChannel(
  //       partyA,
  //       partyA,
  //       partyB,
  //       depositAmount
  //     );

  //     await hoprChannels.initiateChannelClosure(partyB, {
  //       from: partyA
  //     });

  //     await expectRevert(
  //       hoprChannels.claimChannelClosure(partyB, {
  //         from: randomUser
  //       }),
  //       "channel is not pending for closure"
  //     );
  //   });

  //   it("should fail 'createChannel' when token balance too low'", async function() {
  //     await hoprToken.approve(hoprChannels.address, depositAmount);
  //     await hoprToken.burn(totalSupply, {
  //       from: partyA
  //     });

  //     await expectRevert(
  //       hoprChannels.createChannel(partyA, partyA, partyB, depositAmount, {
  //         from: partyA
  //       }),
  //       "SafeERC20: low-level call failed"
  //     );
  //   });
  // });
});
