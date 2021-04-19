import assert from 'assert'
import { durations, stringToU8a } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { getContracts, migrate, fund } from '@hoprnet/hopr-ethereum'
import HoprEthereum from '.'
import { computeWinningProbability } from './utils'
import { UnacknowledgedTicket, Ticket, Hash } from './types'
import * as testconfigs from './config.spec'
import { createNode } from './utils/testing'

const FUND_ARGS = `--address ${getContracts().localhost.HoprToken.address} --accounts-to-fund 1`

// TODO: replace legacy test
describe('test hashedSecret', function () {
  this.timeout(durations.minutes(10))
  const ganache = new Ganache()
  let connector: HoprEthereum

  // instead of using a half-assed mock we use the connector instance
  // the whole test needs to be rewritten
  async function generateConnector(): Promise<HoprEthereum> {
    const privKey = stringToU8a(testconfigs.DEMO_ACCOUNTS[0])
    return createNode(privKey, 0)
  }

  describe('random pre-image', function () {
    this.timeout(durations.minutes(2))

    before(async function () {
      this.timeout(durations.minutes(1))
      await ganache.start()
      await migrate()
      await fund(FUND_ARGS)

      connector = await generateConnector()
    })

    after(async function () {
      await connector.stop()
      await ganache.stop()
    })

    it('should publish a hashed secret', async function () {
      await connector.hashedSecret.initialize()

      let onChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.accounts(connector.account.address.toHex())).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage)
      assert(preImage.hash().eq(onChainHash))

      await (
        await connector.account.sendTransaction(connector.hoprChannels.updateAccountSecret, preImage.toHex())
      ).wait()

      let updatedOnChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.accounts(connector.account.address.toHex())).secret)
      )

      assert(!onChainHash.eq(updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!preImage.eq(updatedPreImage), `new and old pre-image must not be the same`)

      assert(updatedPreImage.hash().eq(updatedOnChainHash))
    })
  })
})
