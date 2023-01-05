import { PRG as Rust_PRG, PRP as Rust_PRP, SharedKeys } from './cryptography.js'
import { generateKeyShares, PRG as TS_PRG, PRP as TS_PRP, u8aToHex } from '@hoprnet/hopr-utils'
import assert from 'assert'

import { peerIdFromString } from '@libp2p/peer-id'

describe('cryptographic correspondence tests', async function () {
  it('PRG correspondence', async function () {
    let key = new Uint8Array(16)
    let iv = new Uint8Array(12)

    {
      let rs_output = new Rust_PRG(key, iv).digest(5, 10)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(5, 10)
      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_output = new Rust_PRG(key, iv).digest(0, 100)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(0, 100)
      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_output = new Rust_PRG(key, iv).digest(10, 100)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(10, 100)
      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_output = new Rust_PRG(key, iv).digest(16, 22)
      let ts_output = TS_PRG.createPRG({ key, iv }).digest(16, 22)
      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }
  })

  it('PRP correspondence - same ciphertext', async function () {
    let key = new Uint8Array(128)
    let iv = new Uint8Array(64)

    let ts_prp = TS_PRP.createPRP({ key, iv })
    let rs_prp = new Rust_PRP(key, iv)

    let pt = new Uint8Array(100)
    let ct_1 = rs_prp.forward(pt)
    let ct_2 = ts_prp.permutate(pt)

    assert.equal(u8aToHex(ct_1), u8aToHex(ct_2))
  })

  it('PRP correspondence - TS forward / RS inverse', async function () {
    let key = new Uint8Array(128)
    let iv = new Uint8Array(64)

    let ts_prp = TS_PRP.createPRP({ key, iv })
    let rs_prp = new Rust_PRP(key, iv)

    let pt_1 = new Uint8Array(100)
    let ct = ts_prp.permutate(pt_1)
    let pt_2 = rs_prp.inverse(ct)

    assert.equal(u8aToHex(new Uint8Array(100)), u8aToHex(pt_2))
  })

  it('PRP correspondence - RS forward / TS inverse', async function () {
    let key = new Uint8Array(128)
    let iv = new Uint8Array(64)

    let ts_prp = TS_PRP.createPRP({ key, iv })
    let rs_prp = new Rust_PRP(key, iv)

    let pt_1 = new Uint8Array(100)
    let ct = rs_prp.forward(pt_1)
    let pt_2 = ts_prp.inverse(ct)

    assert.equal(u8aToHex(pt_1), u8aToHex(pt_2))
  })

  it('generate keyshares correspondence', async function () {
    //let peerIds = [0, 1, 2].map(async (_) => await createSecp256k1PeerId());
    let peerIds = [
      '16Uiu2HAm15SBTjbZURUZp139uaBAUtw8uS9gDBhFMMX65iHNo4z9',
      '16Uiu2HAmK6qfNEb5BNKUuTrRSJERuritCSwag3NdQsGyt3JJPyA2',
      '16Uiu2HAmNA49JtveyXGTK1StvF25NeAt6rCbjKH1ahHJGNLnzj33'
    ].map((p) => peerIdFromString(p))

    let keyshares_ts = generateKeyShares(peerIds)

    let pub_keys = peerIds.map((p) => (p.publicKey as Uint8Array).slice(4))
    let keyshares_rs = SharedKeys.generate(pub_keys)

    assert.equal(u8aToHex(keyshares_ts.alpha), u8aToHex(keyshares_rs.get_alpha()))

    assert.equal(keyshares_ts.secrets.length, keyshares_rs.count_shared_keys())

    for (let i = 0; i < keyshares_rs.count_shared_keys(); i++) {
      assert.equal(u8aToHex(keyshares_ts.secrets[i]), u8aToHex(keyshares_rs.get_peer_shared_key(i)))
    }
  })
})
