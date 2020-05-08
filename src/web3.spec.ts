import assert from 'assert'
import Web3 from 'web3'
import { Ganache } from '@hoprnet/hopr-ethereum'
import { time } from './utils'
import * as configs from './config'

describe.only('test web3 connect/reconnect', function () {
  const ganache = new Ganache()
  let web3: Web3

  // disconnect provide, mimic behaviour when connection is lost
  const disconnect = async () => {
    try {
      // @ts-ignore
      return web3.currentProvider.disconnect(1012, 'restart')
    } catch (err) {
      // console.error(err)
    }
  }

  beforeEach(async function () {
    await ganache.start()

    web3 = new Web3(
      new Web3.providers.WebsocketProvider(configs.DEFAULT_URI, {
        reconnect: { auto: true, delay: 1000, maxAttempts: 5 },
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
    await disconnect()

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
          disconnect()
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
            disconnect()
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

  it.skip('auto reconnects, broadcasts missed blocks', function () {
    this.timeout(5e3)

    const sub = web3.eth.subscribe('newBlockHeaders')
    let count = 0

    return new Promise(async function (resolve) {
      sub.on('data', async function (result) {
        if (count === 0) {
          // trigger "unexpected" disconnect
          await disconnect()
          await time.advanceBlock(web3)
        } else if (count === 1) {
          // exit point
          assert(result.parentHash)
          resolve()
        }

        count++
      })

      await time.advanceBlock(web3)
    }).finally(() => {
      sub.unsubscribe()
    })
  })
})
