import { BN, time, expectEvent, expectRevert } from '@openzeppelin/test-helpers'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import {
  HoprChannelsContract,
  HoprChannelsInstance,
  HoprTokenContract,
  HoprTokenInstance,
} from '../types/truffle-contracts'
import { recoverSigner, keccak256, Ticket, getChannelId, getParties, Fund } from './utils'
import { PromiseType } from '../types/typescript'

const HoprToken: HoprTokenContract = artifacts.require('HoprToken')
const HoprChannels: HoprChannelsContract = artifacts.require('HoprChannels')

const formatAccount = (res: PromiseType<HoprChannelsInstance['accounts']>) => ({
  hashedSecret: res[0],
  counter: res[1],
})

const formatChannel = (res: PromiseType<HoprChannelsInstance['channels']>) => ({
  deposit: res[0],
  partyABalance: res[1],
  closureTime: res[2],
  stateCounter: res[3],
})

// const PrintGasUsed = (name: string) => (
//   response: Truffle.TransactionResponse
// ) => {
//   console.log(`gas used in '${name}'`, response.receipt.gasUsed);
//   return response;
// };

contract('HoprChannels', function ([accountA, accountB]) {
  const { partyA, partyB } = getParties(accountA, accountB)

  const partyAPrivKey = NODE_SEEDS[1]
  const partyBPrivKey = NODE_SEEDS[0]

  const depositAmount = web3.utils.toWei('1', 'ether')
  let hoprToken: HoprTokenInstance
  let hoprChannels: HoprChannelsInstance
  let totalSupply: string

  const reset = async () => {
    hoprToken = await HoprToken.new()
    // mint supply
    await hoprToken.mint(partyA, web3.utils.toWei('100', 'ether'), '0x00', '0x00')
    await hoprToken.mint(partyB, web3.utils.toWei('100', 'ether'), '0x00', '0x00')
    totalSupply = await hoprToken.totalSupply().then((res) => res.toString())

    hoprChannels = await HoprChannels.new(hoprToken.address, time.duration.days(2))
  }

  // integration tests: reset contracts once
  describe('integration tests', function () {
    before(async function () {
      await reset()
    })

    context("make payments between 'partyA' and 'partyB' using a fresh channel and 'fundChannel'", function () {
      const partyASecret1 = keccak256({
        type: 'bytes32',
        value: keccak256({ type: 'string', value: 'partyA secret 1' }),
      })
      const partyASecret2 = keccak256({
        type: 'bytes32',
        value: partyASecret1,
      })

      const partyBSecret1 = keccak256({
        type: 'bytes32',
        value: keccak256({ type: 'string', value: 'partyB secret 1' }),
      })
      const partyBSecret2 = keccak256({
        type: 'bytes32',
        value: partyBSecret1,
      })

      it("'partyA' should fund 'partyA' with 1 HOPR", async function () {
        await hoprToken.approve(hoprChannels.address, totalSupply, {
          from: partyA,
        })

        const result = await hoprChannels.fundChannel(partyA, partyB, depositAmount, {
          from: partyA,
        })

        expectEvent(result, 'FundedChannel', {
          funder: partyA,
          recipient: partyA,
          counterParty: partyB,
          recipientAmount: depositAmount,
          counterPartyAmount: new BN(0),
        })

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(true, 'wrong deposit')
        expect(channel.partyABalance.eq(new BN(depositAmount))).to.be.equal(true, 'wrong partyABalance')
        expect(channel.stateCounter.eq(new BN(1))).to.be.equal(true, 'wrong stateCounter')
      })

      it("'partyB' should fund 'partyB' with 1 HOPR", async function () {
        await hoprToken.approve(hoprChannels.address, totalSupply, {
          from: partyB,
        })

        const result = await hoprChannels.fundChannel(partyB, partyA, depositAmount, {
          from: partyB,
        })

        expectEvent(result, 'FundedChannel', {
          funder: partyB,
          recipient: partyB,
          counterParty: partyA,
          recipientAmount: depositAmount,
          counterPartyAmount: new BN(0),
        })

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))).to.be.equal(true, 'wrong deposit')

        expect(channel.partyABalance.eq(new BN(depositAmount))).to.be.equal(true, 'wrong partyABalance')

        expect(channel.stateCounter.eq(new BN(1))).to.be.equal(true, 'wrong stateCounter')
      })

      it("should set hashed secret for 'partyA'", async function () {
        // make a ticket to generate hashedSecret for 'partyA'
        const ticket = Ticket({
          web3,
          accountA: partyA,
          accountB: partyB,
          signerPrivKey: partyAPrivKey,
          porSecretA: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret a' }),
          }),
          porSecretB: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret b' }),
          }),
          counterPartySecret: partyASecret2,
          amount: web3.utils.toWei('0.2', 'ether'),
          counter: 1,
          winProbPercent: '100',
        })

        await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
          from: partyA,
        })

        const partyAAccount = await hoprChannels.accounts(partyA).then(formatAccount)

        expect(partyAAccount.hashedSecret).to.be.equal(ticket.hashedCounterPartySecret, 'wrong hashedSecret')

        expect(partyAAccount.counter.eq(new BN(1))).to.be.equal(true, 'wrong counter')
      })

      it("should set hashed secret for 'partyB'", async function () {
        // make a ticket to generate hashedSecret for 'partyB'
        const ticket = Ticket({
          web3,
          accountA: partyA,
          accountB: partyB,
          signerPrivKey: partyAPrivKey,
          porSecretA: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret a' }),
          }),
          porSecretB: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret b' }),
          }),
          counterPartySecret: partyBSecret2,
          amount: web3.utils.toWei('0.2', 'ether'),
          counter: 1,
          winProbPercent: '100',
        })

        await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
          from: partyB,
        })

        const partyBAccount = await hoprChannels.accounts(partyB).then(formatAccount)

        expect(partyBAccount.hashedSecret).to.be.equal(ticket.hashedCounterPartySecret, 'wrong hashedSecret')

        expect(partyBAccount.counter.eq(new BN(1))).to.be.equal(true, 'wrong counter')
      })

      it('should open channel', async function () {
        const result = await hoprChannels.openChannel(partyB, {
          from: partyA,
        })

        expectEvent(result, 'OpenedChannel', {
          opener: partyA,
          counterParty: partyB,
        })

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.stateCounter.eq(new BN(2))).to.be.equal(true, 'wrong stateCounter')
      })

      it("'partyA' should reedem winning ticket of 0.2 HOPR", async function () {
        const ticket = Ticket({
          web3,
          accountA: partyA,
          accountB: partyB,
          signerPrivKey: partyBPrivKey,
          porSecretA: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret a' }),
          }),
          porSecretB: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret b' }),
          }),
          counterPartySecret: partyASecret2,
          amount: web3.utils.toWei('0.2', 'ether'),
          counter: 1,
          winProbPercent: '100',
        })

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
        )

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))).to.be.equal(true, 'wrong deposit')

        expect(
          channel.partyABalance.eq(new BN(depositAmount).add(new BN(web3.utils.toWei('0.2', 'ether'))))
        ).to.be.equal(true, 'wrong partyABalance')

        expect(channel.stateCounter.eq(new BN(2))).to.be.equal(true, 'wrong stateCounter')
      })

      it("'partyB' should reedem winning ticket of 1.2 HOPR", async function () {
        const ticket = Ticket({
          web3,
          accountA: partyA,
          accountB: partyB,
          signerPrivKey: partyAPrivKey,
          porSecretA: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret a' }),
          }),
          porSecretB: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret b' }),
          }),
          counterPartySecret: partyBSecret2,
          amount: web3.utils.toWei('1.2', 'ether'),
          counter: 1,
          winProbPercent: '100',
        })

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
        )

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))).to.be.equal(true, 'wrong deposit')

        expect(channel.partyABalance.eq(new BN(0))).to.be.equal(true, 'wrong partyABalance')

        expect(channel.stateCounter.eq(new BN(2))).to.be.equal(true, 'wrong stateCounter')
      })

      it("'partyB' should initiate closure", async function () {
        const result = await hoprChannels.initiateChannelClosure(partyA, {
          from: partyB,
        })

        expectEvent(result, 'InitiatedChannelClosure', {
          initiator: partyB,
          counterParty: partyA,
        })

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.stateCounter.eq(new BN(3))).to.be.equal(true, 'wrong stateCounter')
      })

      it("'partyA' should reedem winning ticket of 0.5 HOPR", async function () {
        const ticket = Ticket({
          web3,
          accountA: partyA,
          accountB: partyB,
          signerPrivKey: partyBPrivKey,
          porSecretA: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret a' }),
          }),
          porSecretB: keccak256({
            type: 'bytes32',
            value: keccak256({ type: 'string', value: 'por secret b' }),
          }),
          counterPartySecret: partyASecret1,
          amount: web3.utils.toWei('0.5', 'ether'),
          counter: 1,
          winProbPercent: '100',
        })

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
        )

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.deposit.eq(new BN(depositAmount).mul(new BN(2)))).to.be.equal(true, 'wrong deposit')

        expect(channel.partyABalance.eq(new BN(web3.utils.toWei('0.5', 'ether')))).to.be.equal(
          true,
          'wrong partyABalance'
        )

        expect(channel.stateCounter.eq(new BN(3))).to.be.equal(true, 'wrong stateCounter')
      })

      it("'partyA' should close channel", async function () {
        await time.increase(time.duration.days(3))

        const result = await hoprChannels.claimChannelClosure(partyB, {
          from: partyA,
        })

        expectEvent(result, 'ClosedChannel', {
          closer: partyA,
          counterParty: partyB,
          partyAAmount: web3.utils.toWei('0.5', 'ether'),
          partyBAmount: web3.utils.toWei('1.5', 'ether'),
        })

        const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

        expect(channel.deposit.eq(new BN(0))).to.be.equal(true, 'wrong deposit')

        expect(channel.partyABalance.eq(new BN(0))).to.be.equal(true, 'wrong partyABalance')

        expect(channel.stateCounter.eq(new BN(10))).to.be.equal(true, 'wrong stateCounter')
      })
    })

    context(
      "make payments between 'partyA' and 'partyB' using a recycled channel and 'fundChannelWithSig'",
      function () {
        const partyASecret1 = keccak256({
          type: 'bytes32',
          value: keccak256({ type: 'string', value: 'partyA secret 2' }),
        })
        const partyASecret2 = keccak256({
          type: 'bytes32',
          value: partyASecret1,
        })

        const partyBSecret1 = keccak256({
          type: 'bytes32',
          value: keccak256({ type: 'string', value: 'partyB secret 2' }),
        })
        const partyBSecret2 = keccak256({
          type: 'bytes32',
          value: partyBSecret1,
        })

        it("'partyA' and 'partyB' should fund a total of 1 HOPR", async function () {
          const totalAmount = web3.utils.toWei('1', 'ether')
          const partyAAmount = web3.utils.toWei('0.2', 'ether')
          const partyBAmount = web3.utils.toWei('0.8', 'ether')

          const notAfter = await time.latest().then((now) => {
            return now.add(time.duration.days(2)).toString()
          })

          const fund = Fund({
            web3,
            stateCounter: '10',
            initiator: partyA,
            deposit: totalAmount,
            partyAAmount: partyAAmount,
            notAfter,
            signerPrivKey: partyBPrivKey,
          })

          const result = await hoprChannels.fundChannelWithSig(
            '10',
            totalAmount,
            partyAAmount,
            notAfter,
            fund.r,
            fund.s,
            fund.v,
            {
              from: partyA,
            }
          )

          expectEvent(result, 'FundedChannel', {
            // funder: partyA,
            recipient: partyA,
            counterParty: partyB,
            recipientAmount: partyAAmount,
            counterPartyAmount: partyBAmount,
          })

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.deposit.eq(new BN(totalAmount))).to.be.equal(true, 'wrong deposit')
          expect(channel.partyABalance.eq(new BN(partyAAmount))).to.be.equal(true, 'wrong partyABalance')
          expect(channel.stateCounter.eq(new BN(11))).to.be.equal(true, 'wrong stateCounter')
        })

        it("should set hashed secret for 'partyA'", async function () {
          // make a ticket to generate hashedSecret for 'partyA'
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret a' }),
            }),
            porSecretB: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret b' }),
            }),
            counterPartySecret: partyASecret2,
            amount: web3.utils.toWei('0.3', 'ether'),
            counter: 2,
            winProbPercent: '100',
          })

          await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
            from: partyA,
          })

          const partyAAccount = await hoprChannels.accounts(partyA).then(formatAccount)

          expect(partyAAccount.hashedSecret).to.be.equal(ticket.hashedCounterPartySecret, 'wrong hashedSecret')

          expect(partyAAccount.counter.eq(new BN(2))).to.be.equal(true, 'wrong counter')
        })

        it("should set hashed secret for 'partyB'", async function () {
          // make a ticket to generate hashedSecret for 'partyB'
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret a' }),
            }),
            porSecretB: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret b' }),
            }),
            counterPartySecret: partyBSecret2,
            amount: web3.utils.toWei('0.7', 'ether'),
            counter: 2,
            winProbPercent: '100',
          })

          await hoprChannels.setHashedSecret(ticket.hashedCounterPartySecret, {
            from: partyB,
          })

          const partyBAccount = await hoprChannels.accounts(partyB).then(formatAccount)

          expect(partyBAccount.hashedSecret).to.be.equal(ticket.hashedCounterPartySecret, 'wrong hashedSecret')

          expect(partyBAccount.counter.eq(new BN(2))).to.be.equal(true, 'wrong counter')
        })

        it('should open channel', async function () {
          const result = await hoprChannels.openChannel(partyB, {
            from: partyA,
          })

          expectEvent(result, 'OpenedChannel', {
            opener: partyA,
            counterParty: partyB,
          })

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.stateCounter.eq(new BN(12))).to.be.equal(true, 'wrong stateCounter')
        })

        it("'partyA' should reedem winning ticket of 0.3 HOPR", async function () {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyBPrivKey,
            porSecretA: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret a' }),
            }),
            porSecretB: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret b' }),
            }),
            counterPartySecret: partyASecret2,
            amount: web3.utils.toWei('0.3', 'ether'),
            counter: 2,
            winProbPercent: '100',
          })

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
          )

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(true, 'wrong deposit')

          expect(channel.partyABalance.eq(new BN(web3.utils.toWei('0.5', 'ether')))).to.be.equal(
            true,
            'wrong partyABalance'
          )

          expect(channel.stateCounter.eq(new BN(12))).to.be.equal(true, 'wrong stateCounter')
        })

        it("'partyB' should reedem winning ticket of 0.5 HOPR", async function () {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyAPrivKey,
            porSecretA: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret a' }),
            }),
            porSecretB: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret b' }),
            }),
            counterPartySecret: partyBSecret2,
            amount: web3.utils.toWei('0.5', 'ether'),
            counter: 2,
            winProbPercent: '100',
          })

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
          )

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(true, 'wrong deposit')

          expect(channel.partyABalance.eq(new BN(0))).to.be.equal(true, 'wrong partyABalance')

          expect(channel.stateCounter.eq(new BN(12))).to.be.equal(true, 'wrong stateCounter')
        })

        it("'partyB' should initiate closure", async function () {
          const result = await hoprChannels.initiateChannelClosure(partyA, {
            from: partyB,
          })

          expectEvent(result, 'InitiatedChannelClosure', {
            initiator: partyB,
            counterParty: partyA,
          })

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.stateCounter.eq(new BN(13))).to.be.equal(true, 'wrong stateCounter')
        })

        it("'partyA' should reedem winning ticket of 1 HOPR", async function () {
          const ticket = Ticket({
            web3,
            accountA: partyA,
            accountB: partyB,
            signerPrivKey: partyBPrivKey,
            porSecretA: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret a' }),
            }),
            porSecretB: keccak256({
              type: 'bytes32',
              value: keccak256({ type: 'string', value: 'por secret b' }),
            }),
            counterPartySecret: partyASecret1,
            amount: web3.utils.toWei('1', 'ether'),
            counter: 2,
            winProbPercent: '100',
          })

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
          )

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(true, 'wrong deposit')

          expect(channel.partyABalance.eq(new BN(depositAmount))).to.be.equal(true, 'wrong partyABalance')

          expect(channel.stateCounter.eq(new BN(13))).to.be.equal(true, 'wrong stateCounter')
        })

        it("'partyB' should close channel", async function () {
          await time.increase(time.duration.days(3))

          const result = await hoprChannels.claimChannelClosure(partyA, {
            from: partyB,
          })

          expectEvent(result, 'ClosedChannel', {
            closer: partyB,
            counterParty: partyA,
            partyAAmount: web3.utils.toWei('1', 'ether'),
            partyBAmount: '0',
          })

          const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

          expect(channel.deposit.eq(new BN(0))).to.be.equal(true, 'wrong deposit')

          expect(channel.partyABalance.eq(new BN(0))).to.be.equal(true, 'wrong partyABalance')

          expect(channel.stateCounter.eq(new BN(20))).to.be.equal(true, 'wrong stateCounter')
        })
      }
    )
  })

  // unit tests: reset contracts for every test
  describe('unit tests', function () {
    beforeEach(async function () {
      await reset()
    })

    it('should have set hashedSecret correctly', async function () {
      const secretHash = keccak256({
        type: 'string',
        value: 'partyB secret',
      })

      const response = await hoprChannels.setHashedSecret(secretHash, {
        from: partyB,
      })

      expectEvent(response, 'SecretHashSet', {
        account: partyB,
        secretHash,
      })
    })

    it("should have funded channel correctly using 'fundChannel'", async function () {
      await hoprToken.approve(hoprChannels.address, depositAmount, {
        from: partyA,
      })

      const result = await hoprChannels.fundChannel(partyA, partyB, depositAmount, {
        from: partyA,
      })

      expectEvent(result, 'FundedChannel', {
        funder: partyA,
        recipient: partyA,
        counterParty: partyB,
        recipientAmount: depositAmount,
        counterPartyAmount: new BN(0),
      })

      // TODO: check balances

      const channel = await hoprChannels.channels(getChannelId(partyA, partyB)).then(formatChannel)

      expect(channel.deposit.eq(new BN(depositAmount))).to.be.equal(true, 'wrong deposit')

      expect(channel.closureTime.isZero()).to.be.equal(true, 'wrong closureTime')

      expect(channel.stateCounter.eq(new BN(1))).to.be.equal(true, 'wrong stateCounter')
    })

    it("ticket 'signer' should be 'partyA'", async function () {
      const ticket = Ticket({
        web3,
        accountA: partyA,
        accountB: partyB,
        signerPrivKey: partyAPrivKey,
        porSecretA: keccak256({
          type: 'bytes32',
          value: keccak256({ type: 'string', value: 'por secret a' }),
        }),
        porSecretB: keccak256({
          type: 'bytes32',
          value: keccak256({ type: 'string', value: 'por secret b' }),
        }),
        counterPartySecret: keccak256({
          type: 'bytes32',
          value: keccak256({ type: 'string', value: 'partyB secret' }),
        }),
        amount: web3.utils.toWei('0.2', 'ether'),
        counter: 1,
        winProbPercent: '100',
      })

      const signer = recoverSigner(web3, ticket.hashedTicket, ticket.signature)
      expect(signer).to.be.eq(partyA, 'wrong signer')
    })

    it("fund 'signer' should be 'partyA'", async function () {
      const fund = Fund({
        web3,
        stateCounter: '0',
        initiator: partyB,
        deposit: depositAmount,
        partyAAmount: depositAmount,
        notAfter: '0',
        signerPrivKey: partyAPrivKey,
      })

      const signer = recoverSigner(web3, fund.hashedFund, fund.signature)
      expect(signer).to.be.eq(partyA, 'wrong signer')
    })

    it('should fail when creating an open channel a second time', async function () {
      await hoprToken.approve(hoprChannels.address, depositAmount)

      await hoprChannels.fundChannel(partyA, partyB, depositAmount)

      await hoprChannels.openChannel(partyB, {
        from: partyA,
      })

      await expectRevert(
        hoprChannels.openChannel(partyB, {
          from: partyA,
        }),
        'channel must be in funded state'
      )
    })

    it("should fail 'fundChannel' when token balance too low'", async function () {
      await hoprToken.approve(hoprChannels.address, depositAmount)
      await hoprToken.burn(await hoprToken.balanceOf(partyA), '0x00', {
        from: partyA,
      })

      await expectRevert(
        hoprChannels.fundChannel(partyA, partyB, depositAmount, {
          from: partyA,
        }),
        'SafeERC20: low-level call failed'
      )
    })

    it("should fail when 'claimChannelClosure' before closureTime", async function () {
      await hoprToken.approve(hoprChannels.address, depositAmount)

      await hoprChannels.fundChannel(partyA, partyB, depositAmount)

      await hoprChannels.openChannel(partyB, {
        from: partyA,
      })

      await hoprChannels.initiateChannelClosure(partyB, {
        from: partyA,
      })

      await expectRevert(
        hoprChannels.claimChannelClosure(partyB, {
          from: partyA,
        }),
        "'closureTime' has not passed"
      )
    })
  })
})
