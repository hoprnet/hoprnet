import { expect } from 'chai'
import NonceTracker, { Transaction } from './nonce-tracker'
import { durations } from '@hoprnet/hopr-utils'

const USER_ADDRESS = '0x7d3517b0d011698406d6e0aed8453f0be2697926'

describe('nonce-tracker', function () {
  let nonceTracker: NonceTracker
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
    pendingTxs = []
    confirmedTxs = []
  })

  it('should create nonces from 1 to 10', async function () {
    for (let i = 0; i < 10; i++) {
      const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
      pendingTxs.push(genTx({ nonce: nonceLock.nextNonce }))
      nonceLock.releaseLock()
    }

    expect(pendingTxs.map((tx) => tx.nonce)).to.deep.equal([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
  })

  it('should create nonces from 5 to 10 because we have 5 confirmed', async function () {
    confirmedTxs = genMultiTx({ fromNonce: 0, count: 5 })

    for (let i = 0; i < 5; i++) {
      const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
      pendingTxs.push(genTx({ nonce: nonceLock.nextNonce }))
      nonceLock.releaseLock()
    }

    expect(pendingTxs.map((tx) => tx.nonce)).to.deep.equal([5, 6, 7, 8, 9])
  })

  it('should create nonce 4 when we have 3 confirmed and 1 pending', async function () {
    confirmedTxs = genMultiTx({ fromNonce: 0, count: 3 })
    pendingTxs = genMultiTx({ fromNonce: 3, count: 1 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()

    expect(nonceLock.nextNonce).to.equal(4)
  })

  it('should create nonce 2 again when nonce 2 has failed', async function () {
    confirmedTxs = genMultiTx({ fromNonce: 0, count: 1 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()
    expect(nonceLock.nextNonce).to.equal(1)
    // add to pending
    pendingTxs.push(genTx({ nonce: nonceLock.nextNonce }))

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

    pendingTxs = genMultiTx({
      fromNonce: 3,
      count: 29
    })

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
    confirmedTxs = genMultiTx({ count: 1 })
    pendingTxs = genMultiTx({
      forceNonce: 1,
      count: 5
    })

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

    confirmedTxs = genMultiTx({ count: 3 })

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

    pendingTxs = genMultiTx({ count: 2 })

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

    pendingTxs = genMultiTx({ count: 2 })

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    expect(nonceLock.nextNonce).to.equal(5, `nonce should be 5 got ${nonceLock.nextNonce}`)
    nonceLock.releaseLock()
  })

  it('should create nonce 5 after those when there are some pending nonces below the remote one and some over.', async function () {
    pendingTxs = genMultiTx({ count: 5 })
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
    genMultiTx({ count: 5 })
    pendingTxs = genMultiTx({ count: 5, fromNonce: 5 })

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
    confirmedTxs = genMultiTx({ count: 10 })
    pendingTxs = genMultiTx({
      forceNonce: 100,
      count: 1
    })
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
    confirmedTxs = genMultiTx({ count: 64 })
    pendingTxs = genMultiTx({ count: 10, fromNonce: 64 })
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

  it('should not ignore long-time pending transactions when minPending is not provided', async function () {
    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: getTransactionCountFromConfirmed,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs
    })

    let createdAt = new Date().getTime() - durations.seconds(30)

    pendingTxs = [
      genTx({ nonce: 0, createdAt }),
      genTx({ nonce: 1, createdAt: (createdAt += durations.seconds(10)) }),
      genTx({ nonce: 2, createdAt: (createdAt += durations.seconds(10)) }),
      genTx({ nonce: 3, createdAt: (createdAt += durations.seconds(10)) }),
      genTx({ nonce: 4, createdAt: (createdAt += durations.seconds(10)) })
    ]

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()
    expect(nonceLock.nextNonce).to.equal(5, `nonce should be 5 got ${nonceLock.nextNonce}`)
  })

  it('should ignore long-time pending transactions', async function () {
    const minPending = durations.seconds(30)

    nonceTracker = new NonceTracker({
      getLatestBlockNumber: async () => 1,
      getTransactionCount: getTransactionCountFromConfirmed,
      getPendingTransactions: () => pendingTxs,
      getConfirmedTransactions: () => confirmedTxs,
      minPending
    })

    let createdAt = new Date().getTime() - minPending

    pendingTxs = [
      genTx({ nonce: 0, createdAt }),
      genTx({ nonce: 1, createdAt: (createdAt += durations.seconds(10)) }),
      genTx({ nonce: 2, createdAt: (createdAt += durations.seconds(10)) }),
      genTx({ nonce: 3, createdAt: (createdAt += durations.seconds(10)) }),
      genTx({ nonce: 4, createdAt: (createdAt += durations.seconds(10)) })
    ]

    const nonceLock = await nonceTracker.getNonceLock(USER_ADDRESS)
    nonceLock.releaseLock()
    expect(nonceLock.nextNonce).to.equal(0, `nonce should be 0 got ${nonceLock.nextNonce}`)
  })
})

const genTx = (opts: { nonce: number; createdAt?: number }): Transaction => {
  const { createdAt = new Date().getTime() } = opts
  return { ...opts, from: USER_ADDRESS, createdAt }
}

const genMultiTx = (opts: {
  count?: number
  fromNonce?: number
  forceNonce?: number
  createdAt?: number
}): Transaction[] => {
  const { count = 1, fromNonce = 0, forceNonce, createdAt } = opts
  const txs: Transaction[] = []

  for (let i = 0; i < count; i++) {
    txs.push(genTx({ nonce: forceNonce ?? fromNonce + i, createdAt }))
  }

  return txs
}
