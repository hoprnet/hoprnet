import NonceTracker, { Transaction } from './nonce-tracker'
import { expect } from 'chai'

const USER_ADDRESS = '0x7d3517b0d011698406d6e0aed8453f0be2697926'

describe('nonce-tracker', function () {
  let nonceTracker: NonceTracker
  let mockTxGen: MockTxGen
  let pendingTxs: Transaction[] = []
  let confirmedTxs: Transaction[] = []

  const getTransactionCountFromConfirmed = async () => {
    const nonces = confirmedTxs.map((txMeta) => txMeta.nonce)
    if (nonces.length === 0) return 0

    const highestNonce = Math.max(...nonces)
    return highestNonce
  }

  beforeEach(function () {
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: getTransactionCountFromConfirmed,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })
    mockTxGen = new MockTxGen()
    pendingTxs = []
    confirmedTxs = []
  })

  it('should create nonces from 1 to 10', async function () {
    for (let i = 0; i < 10; i++) {
      const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
      pendingTxs.push(mockTxGen.generate({ nonce: nonceLock.nextNonce }))
      nonceLock.releaseLock()
    }

    expect(pendingTxs.map((tx) => tx.nonce)).to.deep.equal([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
  })

  it('should create nonces from 5 to 10 because we have 5 confirmed', async function () {
    confirmedTxs = mockTxGen.generateMulti({}, { fromNonce: 0, count: 5 })

    for (let i = 0; i < 5; i++) {
      const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
      pendingTxs.push(mockTxGen.generate({ nonce: nonceLock.nextNonce }))
      nonceLock.releaseLock()
    }

    expect(pendingTxs.map((tx) => tx.nonce)).to.deep.equal([5, 6, 7, 8, 9])
  })

  it('should create nonce 4 when we have 3 confirmed and 1 pending', async function () {
    confirmedTxs = mockTxGen.generateMulti({}, { fromNonce: 0, count: 3 })
    pendingTxs = mockTxGen.generateMulti({}, { fromNonce: 3, count: 1 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()

    expect(nonceLock.nextNonce).to.equal(4)
  })

  it('should create nonce 2 again when nonce 2 has failed', async function () {
    confirmedTxs = mockTxGen.generateMulti({}, { fromNonce: 0, count: 1 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()
    expect(nonceLock.nextNonce).to.equal(1)
    // add to pending
    pendingTxs.push(mockTxGen.generate({ nonce: nonceLock.nextNonce }))

    // next nonce should be 2
    const secondNonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    secondNonceLock.releaseLock()
    expect(secondNonceLock.nextNonce).to.equal(2)

    // earlier transaction has failed
    pendingTxs.pop()

    const thirdNonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    thirdNonceLock.releaseLock()
    expect(thirdNonceLock.nextNonce).to.equal(1)
  })

  it('should create nonce 32 when we dont have confirmed txs', async function () {
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 3,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    pendingTxs = mockTxGen.generateMulti(
      {},
      {
        fromNonce: 3,
        count: 29
      }
    )

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()
    expect(nonceLock.nextNonce).to.equal(32, `nonce should be 32 got ${nonceLock.nextNonce}`)
  })

  it('should create nonce 0', async function () {
    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()
    expect(nonceLock.nextNonce).to.equal(0, `nonce should be 0 returned ${nonceLock.nextNonce}`)
  })

  it('should create nonce 2 when duplicate nonces exist', async function () {
    confirmedTxs = mockTxGen.generateMulti({}, { count: 1 })
    pendingTxs = mockTxGen.generateMulti(
      {
        nonce: 1
      },
      { count: 5 }
    )

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.be.equal(2, `nonce should be 2 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 3 when local confirmed count is higher than network nonce', async function () {
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 1,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    confirmedTxs = mockTxGen.generateMulti({}, { count: 3 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(3, `nonce should be 3 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 2 when local pending count is higher than other metrics', async function () {
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 1,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    pendingTxs = mockTxGen.generateMulti({}, { count: 2 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(2, `nonce should be 2 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 5 after those when provider nonce is higher than other metrics', async function () {
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 5,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    pendingTxs = mockTxGen.generateMulti({}, { count: 2 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(5, `nonce should be 5 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 5 after those when there are some pending nonces below the remote one and some over.', async function () {
    pendingTxs = mockTxGen.generateMulti({}, { count: 5 })
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 3,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(5, `nonce should be 5 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create 0 nonce after network nonce when there are pending nonces non sequentially over the network nonce', async function () {
    mockTxGen.generateMulti({}, { count: 5 })
    // 5 over that number
    pendingTxs = mockTxGen.generateMulti({}, { count: 5 })
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 0,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(0, `nonce should be 0 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 50 after network nonce When all three return different values', async function () {
    confirmedTxs = mockTxGen.generateMulti({}, { count: 10 })
    pendingTxs = mockTxGen.generateMulti(
      {
        nonce: 100
      },
      { count: 1 }
    )
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 50,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(50, `nonce should be 50 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 74 after network nonce', async function () {
    confirmedTxs = mockTxGen.generateMulti({}, { count: 64 })
    pendingTxs = mockTxGen.generateMulti({}, { count: 10 })
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: async () => 64,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(74, `nonce should be 74 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })
})

class MockTxGen {
  private txs: Transaction[] = []

  public generate(tx: Partial<Transaction> = {}): Transaction {
    const nonce = tx.nonce ?? 0
    return { from: USER_ADDRESS, ...tx, nonce, createdAt: new Date().getTime() }
  }

  public generateMulti(
    tx: Partial<Transaction> = {},
    opts: { count?: number; fromNonce?: number } = {}
  ): Transaction[] {
    const { count = 1, fromNonce } = opts
    const txs = []
    let nonce = fromNonce || this.txs.length

    for (let i = 0; i < count; i++) {
      txs.push(this.generate({ ...tx, nonce: tx.nonce ?? nonce }))
      nonce += 1
    }

    this.txs = this.txs.concat(txs)
    return txs
  }
}
