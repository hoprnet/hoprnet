import assert from 'assert'
import { ProbabilisticPayments } from './hashedSecret'
import { OffChainSecret } from './dbKeys'
import type { LevelUp } from 'levelup'
import sinon from 'sinon'

async function generateMocks() {
  let data = {}
  var mockDBBatch = {
    put: (k, v) => {
      data[k] = v
      return mockDBBatch
    },
    write: sinon.fake()
  }

  const mockDb = ({
    get: sinon.fake((k) => Promise.resolve(data[k])),
    put: sinon.fake((k, v) => (data[k] = v)),
    batch: sinon.fake.returns(mockDBBatch),
    __data: data
  } as unknown) as LevelUp

  const mockPrivKey = new Uint8Array()
  const mockStore = sinon.fake()
  const mockFind = sinon.fake()
  const mockRedeem = sinon.fake()
  return { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem }
}

describe('test probabilistic payments', function () {
  it('initialize with no secret on chain or off', async function () {
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()

    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem, 100, 10)
    assert(mockStore.notCalled, 'store not called until initialized')
    await pp.initialize()
    assert(mockFind.called, 'find was called to verify on chain secret')
    assert(mockStore.called, 'store on chain secret was called')
    assert(pp.getOnChainSecret(), 'on chain secret is set')
    assert(mockStore.firstCall.firstArg === pp.getOnChainSecret(), 'onchain secret matches')
    assert(
      await pp.__test_isValidIteratedHash(pp.getOnChainSecret()),
      'on chain secret is a valid iterated hash of the offChainSecret'
    )
    // @ts-ignore
    assert(Object.keys(mockDb.__data).length > 10, 'store multiple pre images')
  })

  it('initialize with secret on chain, but no offchain secret', async function () {
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()
    mockFind = sinon.fake.returns(Promise.resolve(new Uint8Array(10).fill(1)))

    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem, 100, 10)
    assert(mockStore.notCalled, 'store not called until initialized')
    await pp.initialize()
    assert(mockFind.called, 'find was called to verify on chain secret')
    // @ts-ignore
    assert(mockDb.get.called, 'db get was called')
    assert(mockStore.called, 'store on chain secret was called, as it needs to reinitialize')
    assert(pp.getOnChainSecret(), 'on chain secret is set')
    assert(mockStore.firstCall.firstArg === pp.getOnChainSecret(), 'onchain secret matches')
    assert(
      await pp.__test_isValidIteratedHash(pp.getOnChainSecret()),
      'on chain secret is a valid iterated hash of the offChainSecret'
    )
  })

  it('initialize with secret on chain, but does not match offChain', async function () {
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()
    mockFind = sinon.fake.returns(Promise.resolve(new Uint8Array(10).fill(1)))
    await mockDb.put(Buffer.from(OffChainSecret()), Buffer.from(new Uint8Array(10).fill(2)))
    // @ts-ignore
    assert(await mockDb.get(Buffer.from(OffChainSecret())), 'mock was set')

    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem, 100, 10)
    assert(mockStore.notCalled, 'store not called until initialized')
    await pp.initialize()
    assert(mockFind.called, 'find was called to verify on chain secret')
    // @ts-ignore
    assert(mockDb.get.called, 'db get was called')
    assert(mockStore.called, 'store on chain secret was called, as it needs to reinitialize')
    assert(pp.getOnChainSecret(), 'on chain secret is set')
    assert(mockStore.firstCall.firstArg === pp.getOnChainSecret(), 'onchain secret matches')
    assert(
      await pp.__test_isValidIteratedHash(pp.getOnChainSecret()),
      'on chain secret is now a valid iterated hash of the offChainSecret'
    )
  })

  it('initialize with secret on chain and offchain', async function () {
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()
    // First let's get valid onchain and offchain values, by initializing a
    // temporary ProbabilisticPayments
    const temp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem, 100, 10)
    await temp.initialize()
    //const offChain = await mockDb.get(Buffer.from(OffChainSecret()))
    const onChain = temp.getOnChainSecret()
    mockFind = sinon.fake.returns(Promise.resolve(onChain))
    // Reset mocks
    mockStore.resetHistory()

    // Now we create a ProbablisticPayments with valid db and onchainsecret
    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem, 100, 10)
    await pp.initialize()
    assert(mockStore.notCalled, 'store on chain secret was not called - no reinit')
  })

  it('issue valid 100% tickets', async function(){
    /*
    let { mockDb, mockPrivKey, mockStore, mockFind, mockRedeem } = await generateMocks()
    let pp = new ProbabilisticPayments(mockDb, mockPrivKey, mockStore, mockFind, mockRedeem, 100, 10)

    let ticket = await pp.issueTicket(
      amount, counterparty, challenge, epoch, channelIteration, 1)
    assert(ticket, 'ticket created')
      */
  })



  /*

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
