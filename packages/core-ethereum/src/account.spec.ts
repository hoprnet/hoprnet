import type { HoprToken } from './tsc/web3/HoprToken'
import type { HoprChannels } from './tsc/web3/HoprChannels'
import type { Await } from './tsc/utils'
import type CoreConnector from '.'
import assert from 'assert'
import Web3 from 'web3'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, addresses, abis } from '@hoprnet/hopr-ethereum'
import { stringToU8a, durations } from '@hoprnet/hopr-utils'
import { getPrivKeyData, createAccountAndFund, createNode, disconnectWeb3 } from './utils/testing.spec'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { wait } from './utils'

const HoprTokenAbi = abis.HoprToken
const HoprChannelsAbi = abis.HoprChannels

describe('test Account class', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let web3: Web3
  let hoprToken: HoprToken
  let hoprChannels: HoprChannels
  let coreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>
  let user: Await<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses.localhost?.HoprToken)
    hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, addresses.localhost?.HoprChannels)
  })

  after(async function () {
    await ganache.stop()
  })

  beforeEach(async function () {
    this.timeout(durations.minutes(1))
    funder = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    user = await createAccountAndFund(web3, hoprToken, funder, testconfigs.DEMO_ACCOUNTS[1])
    coreConnector = await createNode(user.privKey, false)

    // wait until it starts
    await coreConnector.start()
    await coreConnector.initOnchainValues()
  })

  afterEach(async function () {
    await coreConnector.stop()
  })

  describe('ticketEpoch', function () {
    it('should be 1 initially', async function () {
      const ticketEpoch = await coreConnector.account.ticketEpoch

      assert.equal(ticketEpoch.toString(), '1', 'initial ticketEpoch is wrong')
    })

    it('should be 2 after setting new secret', async function () {
      const ticketEpoch = await coreConnector.account.ticketEpoch

      assert.equal(ticketEpoch.toString(), '2', 'ticketEpoch is wrong')
    })

    it('should be 3 after reconnecting to web3', async function () {
      this.timeout(durations.seconds(10))
      await disconnectWeb3(coreConnector.web3)

      // wait for reconnection
      await wait(durations.seconds(2))

      const ticketEpoch = await coreConnector.account.ticketEpoch

      assert.equal(ticketEpoch.toString(), '3', 'ticketEpoch is wrong')
    })
  })
})
