import assert from 'assert'
import { ProbabilisticPayments } from './hashedSecret'
//import { u8aEquals, stringToU8a } from '@hoprnet/hopr-utils'
//import { hash as hashFunction } from './utils'
import type { LevelUp } from 'levelup'
import sinon from 'sinon'

async function generateMocks(){
  let data = {}
  var mockDBBatch = {
    put: (k, v) => {
      data[k] = v
      return mockDBBatch
    },
    write: sinon.fake()
  }

  const mockDb = {
    get: sinon.fake((k) => data[k]),
    batch: sinon.fake.returns(mockDBBatch),
    __data: data
  } as unknown as LevelUp

  const mockPrivKey = new Uint8Array()
  const mockStore = sinon.fake()
  const mockFind = sinon.fake()
  const mockRedeem = sinon.fake()
  return { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem }
}

describe('test probabilistic payments', function () {

  it('initialize with no secret on chain or off', async function() {
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()

    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem)
    assert(mockStore.notCalled, 'store not called until initialized')
    await pp.initialize()
    assert(mockFind.called, 'find was called to verify on chain secret')
    assert(mockStore.called, 'store on chain secret was called')
    assert(pp.getOnChainSecret(), 'on chain secret is set')
    assert(mockStore.firstCall.firstArg === pp.getOnChainSecret(), 'onchain secret matches')
    assert(pp.__test_isValidIteratedHash(pp.getOnChainSecret()),
           'on chain secret is a valid iterated hash of the offChainSecret')
    // @ts-ignore
    assert(Object.keys(mockDb.__data).length > 10, 'store multiple pre images')
  })

  it('initialize with secret on chain, but no offchain secret', async function(){
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()
    mockFind = sinon.fake.returns(Promise.resolve(new Uint8Array(10).fill(1)))

    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem)
    assert(mockStore.notCalled, 'store not called until initialized')
    await pp.initialize()
    assert(mockFind.called, 'find was called to verify on chain secret')
    // @ts-ignore
    assert(mockDb.get.called, 'db get was called')
    assert(mockStore.called, 'store on chain secret was called, as it needs to reinitialize')
    assert(pp.getOnChainSecret(), 'on chain secret is set')
    assert(mockStore.firstCall.firstArg === pp.getOnChainSecret(), 'onchain secret matches')
    assert(pp.__test_isValidIteratedHash(pp.getOnChainSecret()),
           'on chain secret is a valid iterated hash of the offChainSecret')
  })

/*
  it('initialize with secret on chain, but does not match offChain', function(){
    let probabilisticPayments = new ProbabilisticPayments(mockDb, mockAccount, mockChannels)
    probabilisticPayments.initialize()
    assert(probabilisticPayments.getOnChainSecret(), 'on chain secret is set')
  })
*/

  /*
      assert(!u8aEquals(onChainHash, updatedOnChainHash), `new and old onChainSecret must not be the same`)
      assert(!u8aEquals(preImage, updatedPreImage), `new and old pre-image must not be the same`)
    })
  })
  describe('deterministic debug pre-image', function () {

    it('should publish a hashed secret', async function () {
      await connector.probabilisticPayments.initialize()

      let onChainHash = new Types.Hash(
        stringToU8a(
          (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
        )
      )

      let preImage = await connector.probabilisticPayments.findPreImage(onChainHash)

      assert(u8aEquals((await hashFunction(preImage)).slice(0, HASHED_SECRET_WIDTH), onChainHash))

      await connector.utils.waitForConfirmation(
        (
          await connector.account.signTransaction(
            {
              from: (await connector.account.address).toHex(),
              to: connector.hoprChannels.options.address
            },
            connector.hoprChannels.methods.setHashedSecret(new Types.Hash(preImage).toHex())
          )
        ).send()
      )

      let updatedOnChainHash = new Types.Hash(
        stringToU8a(
          (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
        )
      )

      assert(!u8aEquals(onChainHash, updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.probabilisticPayments.findPreImage(updatedOnChainHash)

      assert(!u8aEquals(preImage, updatedPreImage), `new and old pre-image must not be the same`)

      assert(u8aEquals((await hashFunction(updatedPreImage)).slice(0, HASHED_SECRET_WIDTH), updatedOnChainHash))
    })

    it('should reserve a preImage for tickets with 100% winning probabilty but should not reserve for 0% winning probability', async function () {

      assert(
        (await connector.probabilisticPayments.validateTicket(
          {
            ticket: {
              hash: Promise.resolve(new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))),
              challenge: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff)),
              winProb: Utils.computeWinningProbability(1)
            }
          } as Types.SignedTicket,
          new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))
        )).status === 'SUCCESS',
        'ticket with 100% winning probability must always be a win'
      )

      assert(
        (await connector.probabilisticPayments.validateTicket(
          {
            ticket: {
              hash: Promise.resolve(new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))),
              challenge: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff)),
              winProb: Utils.computeWinningProbability(0)
            }
          } as Types.SignedTicket,
          new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))
        )).status === 'E_TICKET_FAILED',
        'falsy ticket should not be a win'
      )
    })
  })
*/
})
