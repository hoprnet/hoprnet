import assert from 'assert'
import PeerId from 'peer-id'
import {
  isSecp256k1PeerId,
  convertPubKeyFromPeerId,
  convertPubKeyFromB58String,
  hasB58String,
  getB58String,
  libp2pSubscribe,
  libp2pSendMessage
} from '.'
import type { LibP2PHandlerArgs } from './index'
import BL from 'bl'
import { defer } from '../async'
import { u8aEquals } from '../u8a'
import { Multiaddr } from 'multiaddr'

describe(`test convertPubKeyFromPeerId`, function () {
  it(`should equal to a newly created pubkey from PeerId`, async function () {
    const id = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    const pubKey = await convertPubKeyFromPeerId(id)
    assert(id.pubKey.toString() === pubKey.toString())
  })
  it(`should equal to pubkey from a PeerId CID`, async function () {
    const testIdB58String = '16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'
    const pubKey = await convertPubKeyFromB58String(testIdB58String)
    const id = PeerId.createFromB58String(testIdB58String)
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

describe(`test libp2pSendMessage`, function () {
  it(`send message`, async function () {
    const msgToReceive = new TextEncoder().encode(`This message should be received.`)

    const desintation = await PeerId.create({ keyType: 'secp256k1' })
    const msgReceived = defer<void>()

    const fakeLibp2p = {
      dialProtocol(destination: Multiaddr, protocol: string, ..._opts: any[]): Promise<LibP2PHandlerArgs> {
        return Promise.resolve({
          stream: {
            sink: async (source: AsyncIterable<Uint8Array>) => {
              for await (const msg of source) {
                if (u8aEquals(Uint8Array.from(msg.slice()), msgToReceive)) {
                  msgReceived.resolve()
                } else {
                  msgReceived.reject()
                }
              }
            },
            source: []
          } as any,
          protocol,
          connection: {
            remotePeer: destination
          } as any
        })
      },
      peerStore: {
        addressBook: {
          get(_peer: PeerId) {
            return [
              {
                multiaddr: new Multiaddr(`/ip4/1.2.3.4/`)
              }
            ]
          }
        }
      }
    }

    await libp2pSendMessage(fakeLibp2p as any, desintation, 'demo protocol', msgToReceive, false, { timeout: 5000 })

    await msgReceived.promise
  })
})

describe(`test libp2pSendMessage with response`, function () {
  it(`send message and get response`, async function () {
    const msgToReceive = new TextEncoder().encode(`This message should be received.`)
    const msgToReplyWith = new TextEncoder().encode(`This message should be received.`)

    const desintation = await PeerId.create({ keyType: 'secp256k1' })

    const msgReceived = defer<void>()

    const waitUntilSend = defer<void>()

    const fakeLibp2p = {
      dialProtocol(destination: Multiaddr, protocol: string, ..._opts: any[]): Promise<LibP2PHandlerArgs> {
        return Promise.resolve({
          stream: {
            sink: async (source: AsyncIterable<Uint8Array>) => {
              for await (const msg of source) {
                if (u8aEquals(Uint8Array.from(msg.slice()), msgToReceive)) {
                  msgReceived.resolve()
                } else {
                  msgReceived.reject()
                }

                await new Promise((resolve) => setTimeout(resolve, 50))
                waitUntilSend.resolve()
              }
            },
            source: (async function* () {
              await waitUntilSend.promise

              yield new BL(msgToReplyWith as any)
            })()
          } as any,
          protocol,
          connection: {
            remotePeer: destination
          } as any
        })
      },
      peerStore: {
        addressBook: {
          get(_peer: PeerId) {
            return [
              {
                multiaddr: new Multiaddr(`/ip4/1.2.3.4/`)
              }
            ]
          }
        }
      }
    }

    const results = await Promise.all([
      msgReceived.promise,
      await libp2pSendMessage(fakeLibp2p as any, desintation, 'demo protocol', msgToReceive, true, {
        timeout: 5000
      })
    ])

    assert(u8aEquals(results[1][0], msgToReplyWith), `Replied message should match the expected value`)
  })
})

describe(`test libp2pSubscribe`, async function () {
  it(`subscribe and reply`, async function () {
    const msgToReceive = new TextEncoder().encode(`This message should be received.`)
    const msgToReplyWith = new TextEncoder().encode(`This message should be replied by the handler.`)

    const remotePeer = await PeerId.create({ keyType: 'secp256k1' })

    let msgReceived = defer<void>()
    let msgReplied = defer<void>()

    const fakeOnMessage = async (msg: Uint8Array): Promise<Uint8Array> => {
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

    libp2pSubscribe(fakeLibp2p as any, 'demo protocol', fakeOnMessage, () => {}, true)

    await Promise.all([msgReceived.promise, msgReplied.promise])
  })

  it(`subscribe and consume`, async function () {
    const msgToReceive = new TextEncoder().encode(`This message should be received.`)

    const remotePeer = await PeerId.create({ keyType: 'secp256k1' })

    let msgReceived = defer<void>()

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

    libp2pSubscribe(fakeLibp2p as any, 'demo protocol', fakeOnMessage, () => {}, false)

    await msgReceived.promise
  })
})

describe('test libp2p utils', function () {
  it('should be a secp256k1 peerId', async function () {
    const pId = await PeerId.create({ keyType: 'secp256k1' })

    assert(isSecp256k1PeerId(pId) == true, 'peerId must have a secp256k1 keypair')

    const pIdDifferentKey = await PeerId.create({ keyType: 'Ed25519' })

    assert(isSecp256k1PeerId(pIdDifferentKey) == false, 'peerId does not have a secp256k1 keypair')
  })
})
