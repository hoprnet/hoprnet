import assert from 'assert'
import Web3 from 'web3'
import { Ganache } from '@hoprnet/hopr-testing'
import { time } from './utils'
import { disconnectWeb3 } from './utils/testing.spec'
import * as configs from './config'

describe('test web3 connect/reconnect', function () {
  const ganache = new Ganache()
  let web3: Web3

  beforeEach(async function () {
    await ganache.start()

    web3 = new Web3(
      new Web3.providers.WebsocketProvider(configs.DEFAULT_URI, {
        reconnect: { auto: true, delay: 500, maxAttempts: 2, onTimeout: true }
      })
    )
  })

  afterEach(async function () {
    await ganache.stop()
  })

  it('queues requests made while connection is lost / executes on reconnect', async function () {
    this.timeout(5e3)

    // make request
    const deferred = web3.eth.getBlockNumber()

    // disconnect
    await disconnectWeb3(web3)

    // wait for reponse
    const blockNumber = await deferred
    assert(blockNumber === 0)
  })

  it('auto reconnects, keeps the subscription running and triggers the `connected` event listener twice', function () {
    this.timeout(5e3)

    const sub = web3.eth.subscribe('newBlockHeaders')
    let count = 0

    return new Promise(function (resolve) {
      sub.on('connected', function (result) {
        if (count === 0) {
          // trigger "unexpected" disconnect
          disconnectWeb3(web3)
        } else if (count === 1) {
          // exit point
          assert(result)
          resolve()
        }

        count++
      })
    }).finally(() => {
      sub.unsubscribe()
    })
  })

  it('auto reconnects, keeps the subscription running and keeps listening to new blocks', function () {
    this.timeout(5e3)

    const sub = web3.eth.subscribe('newBlockHeaders')
    let count = 0

    return new Promise(function (resolve) {
      sub
        .on('connected', function () {
          // on connect, mine next block
          time.advanceBlock(web3)
        })
        .on('data', function (result) {
          if (count === 0) {
            // trigger "unexpected" disconnect
            disconnectWeb3(web3)
          } else if (count === 1) {
            // exit point
            assert(result.parentHash)
            resolve()
          }

          count++
        })
    }).finally(() => {
      sub.unsubscribe()
    })
  })

  // // after v1.2.8 https://github.com/ethereum/web3.js/pull/3494
  // it('should not emit error while reconnecting', function () {
  //   this.timeout(20e3)

  //   const p = new Promise((resolve) => {
  //     // @ts-ignore
  //     web3.currentProvider.on('error', (err) => {
  //       console.log('here', err)

  //       resolve()
  //     })
  //   })

  //   disconnectWeb3(web3)

  //   return p
  // })

  // // after v1.2.8 https://github.com/ethereum/web3.js/pull/3502
  // it('auto reconnects, broadcasts missed blocks', function () {
  //   this.timeout(5e3)

  //   const sub = web3.eth.subscribe('newBlockHeaders')
  //   let count = 0

  //   return new Promise(async function (resolve) {
  //     sub.on('data', async function (result) {
  //       if (count === 0) {
  //         // trigger "unexpected" disconnect
  //         await disconnectWeb3(web3)
  //         await time.advanceBlock(web3)
  //       } else if (count === 1) {
  //         // exit point
  //         assert(result.parentHash)
  //         resolve()
  //       }

  //       count++
  //     })

  //     await time.advanceBlock(web3)
  //   }).finally(() => {
  //     sub.unsubscribe()
  //   })
  // })
})
