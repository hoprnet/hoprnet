[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [nonce-tracker](../modules/nonce_tracker.md) / NonceLock

# Interface: NonceLock

[nonce-tracker](../modules/nonce_tracker.md).NonceLock

**`property`** nextNonce - The highest of the nonce values derived based on confirmed and pending transactions and eth_getTransactionCount method

**`property`** nonceDetails - details of nonce value derivation.

**`property`** releaseLock

## Table of contents

### Properties

- [nextNonce](nonce_tracker.noncelock.md#nextnonce)
- [nonceDetails](nonce_tracker.noncelock.md#noncedetails)
- [releaseLock](nonce_tracker.noncelock.md#releaselock)

## Properties

### nextNonce

• **nextNonce**: _number_

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:32](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L32)

---

### nonceDetails

• **nonceDetails**: [_NonceDetails_](../modules/nonce_tracker.md#noncedetails)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:33](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L33)

---

### releaseLock

• **releaseLock**: () => _void_

#### Type declaration

▸ (): _void_

**Returns:** _void_

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L34)

Defined in: [packages/core-ethereum/src/nonce-tracker.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/nonce-tracker.ts#L34)
