import assert from 'assert'
import Web3 from 'web3'
import { Ganache, migrate, fund } from '@hoprnet/hopr-ethereum'
import { stringToU8a } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import * as configs from '../config'
import { wait, getParties } from '../utils'
import { getPrivKeyData, generateUser, generateNode } from '../utils/testing'
import { HoprToken } from '../tsc/web3/HoprToken'
import { HoprChannels } from '../tsc/web3/HoprChannels'
import { Await } from '../tsc/utils'
import type CoreConnector from '..'

describe.only('test channels', function () {
  const ganache = new Ganache()
  let web3: Web3
  let hoprToken: HoprToken
  let hoprChannels: HoprChannels
  let coreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>
  let userA: Await<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(60e3)

    // initialize ganache and contracts
    await ganache.start()
    await migrate()
    await fund()

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
    hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, configs.CHANNELS_ADDRESSES.private)
    funder = await getPrivKeyData(stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY))
    userA = await generateUser(web3, funder, hoprToken)
    coreConnector = await generateNode(userA.privKey)

    await hoprToken.methods.approve(hoprChannels.options.address, 100).send({
      from: funder.address.toHex(),
    })

    await coreConnector.db.clear()
  })

  after(async function () {
    // @ts-ignore
    web3.currentProvider.disconnect()
    await ganache.stop()
  })

  it('should store channel & blockNumber correctly', async function () {
    this.timeout(60e3)

    await coreConnector.channels.start(coreConnector)

    await hoprChannels.methods.fundChannel(funder.address.toHex(), userA.address.toHex(), 1).send({
      from: funder.address.toHex(),
      gas: 200e3,
    })

    const blockNumber = await web3.eth.getBlockNumber()

    await hoprChannels.methods.openChannel(userA.address.toHex()).send({
      from: funder.address.toHex(),
      gas: 200e3,
    })

    await wait(2e3)

    const allChannels = await coreConnector.channels.getAll(coreConnector)
    assert.equal(allChannels.length, 1, 'check Channels.store')

    const [partyA, partyB] = getParties(funder.address, userA.address)

    assert(allChannels[0].partyA.eq(partyA), 'check Channels.store')
    assert(allChannels[0].partyB.eq(partyB), 'check Channels.store')
    assert.equal(allChannels[0].blockNumber, blockNumber + 1, 'check Channels.store')
  })
})
