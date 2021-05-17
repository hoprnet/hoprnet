import assert from 'assert'
import PeerId from 'peer-id'
import {
  convertPubKeyFromPeerId,
  convertPubKeyFromB58String,
  hasB58String,
  getB58String,
  libp2pSubscribe,
  LibP2PHandlerArgs
} from '.'
import BL from 'bl'
import Defer from 'p-defer'
import { u8aEquals } from '../u8a'

describe(`test convertPubKeyFromPeerId`, function () {
  it(`should equal to a newly created pubkey from PeerId`, async function () {
    const id = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    const pubKey = await convertPubKeyFromPeerId(id)
    assert(id.pubKey.toString() === pubKey.toString())
  })
  it(`should equal to pubkey from a PeerId CID`, async function () {
    const testIdB58String = '16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'
    const pubKey = await convertPubKeyFromB58String(testIdB58String)
    const id = PeerId.createFromCID(testIdB58String)
    assert(id.pubKey.toString() === pubKey.toString())
  })
})

describe(`test hasB58String`, function () {
  it(`should return a boolean value`, function () {
    const response = hasB58String('test')
    assert(typeof response === 'boolean')
  })
  it(`should return false to a content w/o a b58string`, function () {
    const response = hasB58String('A random string w/o a b58string')
    assert(response === false)
  })
  it(`should return true to a content w/a b58string`, function () {
    const response = hasB58String('16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg')
    assert(response === true)
  })
  it(`should return true to a content w/a b58string`, function () {
    const tweet = `16Uiu2HAkz2s8kLcY7KTSkQBDUmfD8eSgKVnYRt8dLM36jDgZ5Z7d 
@hoprnet
 #HOPRNetwork
    `
    const response = hasB58String(tweet)
    assert(response === true)
  })
})

describe(`test hasB58String`, function () {
  it(`should return a string value`, function () {
    const response = getB58String('test')
    assert(typeof response === 'string')
  })
  it(`should return an empty string to a content w/o a b58string`, function () {
    const response = getB58String('A random string w/o a b58string')
    assert(response === '')
  })
  it(`should return the b58string to a content w/a b58string`, function () {
    const response = getB58String('16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg')
    assert(response === '16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg')
  })
  it(`should return the b58string to a content w/a b58string`, function () {
    const tweet = `16Uiu2HAkz2s8kLcY7KTSkQBDUmfD8eSgKVnYRt8dLM36jDgZ5Z7d 
@hoprnet
 #HOPRNetwork
    `
    const response = getB58String(tweet)
    assert(response === '16Uiu2HAkz2s8kLcY7KTSkQBDUmfD8eSgKVnYRt8dLM36jDgZ5Z7d')
  })
})

describe(`test libp2pSubscribe`, async function () {
  it(`subscribe and reply`, async function () {
    const msgToReceive = new TextEncoder().encode(`This message should be received.`)
    const msgToReplyWith = new TextEncoder().encode(`This message should be replied by the handler.`)

    const remotePeer = await PeerId.create({ keyType: 'secp256k1' })

    let msgReceived = Defer()
    let msgReplied = Defer()

    const fakeOnMessage = async (msg: Uint8Array) => {
      if (u8aEquals(msg, msgToReceive)) {
        msgReceived.resolve()
      } else {
        msgReceived.reject()
      }

      return new Promise((resolve) => setTimeout(resolve, 50, msgToReplyWith))
    }

    const fakeLibp2p = {
      handle(protocols: string[], handlerFunction: (args: LibP2PHandlerArgs) => Promise<void>) {
        handlerFunction({
          stream: {
            source: (async function* () {
              yield new BL(msgToReceive as any)
            })(),
            sink: (async (source: AsyncIterable<Uint8Array>) => {
              const msgs = []
              for await (const msg of source) {
                msgs.push(msg)
              }
              if (msgs.length > 0 && u8aEquals(msgs[0], msgToReplyWith)) {
                msgReplied.resolve()
              } else {
                msgReplied.reject()
              }
            }) as any
          } as any,
          connection: {
            remotePeer
          } as any,
          protocol: protocols[0]
        })
      }
    }

    libp2pSubscribe(fakeLibp2p as any, 'demo protocol', fakeOnMessage, true)

    await Promise.all([msgReceived.promise, msgReplied.promise])
  })

  it(`subscribe and consume`, async function () {
    const msgToReceive = new TextEncoder().encode(`This message should be received.`)

    const remotePeer = await PeerId.create({ keyType: 'secp256k1' })

    let msgReceived = Defer()

    const fakeOnMessage = async (msg: Uint8Array) => {
      await new Promise((resolve) => setTimeout(resolve, 50))

      if (u8aEquals(msg, msgToReceive)) {
        msgReceived.resolve()
      } else {
        msgReceived.reject()
      }
    }

    const fakeLibp2p = {
      handle(protocols: string[], handlerFunction: (args: LibP2PHandlerArgs) => Promise<void>) {
        handlerFunction({
          stream: {
            source: (async function* () {
              yield new BL(msgToReceive as any)
            })(),
            sink: (() => {}) as any
          } as any,
          connection: {
            remotePeer
          } as any,
          protocol: protocols[0]
        })
      }
    }

    libp2pSubscribe(fakeLibp2p as any, 'demo protocol', fakeOnMessage, false)

    await msgReceived.promise
  })
})
