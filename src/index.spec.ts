import assert from 'assert'
import Web3 from 'web3'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-ethereum'
import HoprEthereum from '.'
import { generateNode } from './utils/testing'
import * as configs from './config'

describe('test connector', function () {
  const ganache = new Ganache()
  let web3: Web3
  let connector: HoprEthereum

  before(async function () {
    this.timeout(60e3)

    await ganache.start()

    web3 = new Web3(configs.DEFAULT_URI)
    connector = await generateNode(stringToU8a(configs.DEMO_ACCOUNTS[0]))
  })

  after(async function () {
    // @ts-ignore
    web3.currentProvider.disconnect()
    await ganache.stop()
  })

  context('nonces', function () {
    it('should generate nonces in parallel', async function () {
      const results = await Promise.all(
        Array.from({ length: 5 }).map(async (_, expectedNonce) => {
          const nonce = await connector.nonce
          return nonce === expectedNonce
        })
      )

      assert(
        results.every((r) => r),
        'incorrect nonces'
      )
    })

    it('should generate next nonce', async function () {
      const nonce = await connector.nonce

      assert.equal(nonce, 5, 'incorrect next nonce')
    })
  })
})
