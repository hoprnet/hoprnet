import {
  PRG as Rust_PRG,
  PRGParameters as Rust_PRGParameters,
  PRP as Rust_PRP,
  PRPParameters as Rust_PRPParameters,
  SharedKeys,
  derive_packet_tag
} from './cryptography.js'
import { generateKeyShares, PRG as TS_PRG, PRP as TS_PRP, stringToU8a, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import assert from 'assert'

import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import { forwardTransform } from '@hoprnet/hopr-utils/lib/crypto/packet/keyShares.js'
import {
  derivePacketTag,
  derivePRGParameters,
  derivePRPParameters
} from '@hoprnet/hopr-utils/lib/crypto/packet/keyDerivation.js'

import { SECRET_LENGTH } from '@hoprnet/hopr-utils/lib/crypto/packet/constants.js'

describe('cryptographic correspondence tests', async function () {
  it('derived parameters correspondence', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    let packet_tag_ts = derivePacketTag(secret)
    let packet_tag_rs = derive_packet_tag(secret)
    console.log(u8aToHex(packet_tag_ts))
    assert.equal(u8aToHex(packet_tag_ts), u8aToHex(packet_tag_rs))
  })

  it('PRG correspondence', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = new Rust_PRG(rs_prg_params).digest(5, 10)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(5, 10)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = new Rust_PRG(rs_prg_params).digest(0, 100)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(0, 100)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = new Rust_PRG(rs_prg_params).digest(10, 100)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(10, 100)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = new Rust_PRG(rs_prg_params).digest(16, 22)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(16, 22)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }
  })

  it('PRP correspondence - same ciphertext', async function () {
    //let secret = new Uint8Array(SECRET_LENGTH)

    //let rs_prp_params = new Rust_PRPParameters(secret)
    //let ts_prp_params = derivePRPParameters(secret)

    //assert.equal(u8aToHex(ts_prp_params.key), u8aToHex(rs_prp_params.key()), "keys dont correspond")
    //assert.equal(u8aToHex(ts_prp_params.iv), u8aToHex(rs_prp_params.iv()), "ivs dont correspond")

    //console.log(u8aToHex(rs_prp_params.key()))
    //console.log(u8aToHex(rs_prp_params.iv()))

    //let ts_prp = TS_PRP.createPRP({ key: rs_prp_params.key(), iv: rs_prp_params.iv() })
    let ts_prp = TS_PRP.createPRP({
      key: stringToU8a(
        '0xa9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f35cb715a26e268fc3ce68680e16767581de4e2cb3944c563d1f1a0cc077f3e788a12f31ae07111d77a876a66de5bdd6176bdaa2e07d1cb2e36e428afafdebb2109f70ce8422c8821233053bdd5871523ffb108f1e0f86809999a99d407590df25'
      ),
      iv: stringToU8a(
        '0xa59991716be504b26471dea53d688c4bab8e910328e54ebb6ebf07b49e6d12eacfc56e0935ba2300559b43ede25aa09eee7e8a2deea5f0bdaee2e859834edd38'
      )
    })

    let rs_prp = Rust_PRP.create(
      stringToU8a(
        '0xa9c6632c9f76e5e4dd03203196932350a47562f816cebb810c64287ff68586f35cb715a26e268fc3ce68680e16767581de4e2cb3944c563d1f1a0cc077f3e788a12f31ae07111d77a876a66de5bdd6176bdaa2e07d1cb2e36e428afafdebb2109f70ce8422c8821233053bdd5871523ffb108f1e0f86809999a99d407590df25'
      ),
      stringToU8a(
        '0xa59991716be504b26471dea53d688c4bab8e910328e54ebb6ebf07b49e6d12eacfc56e0935ba2300559b43ede25aa09eee7e8a2deea5f0bdaee2e859834edd38'
      )
    )

    let pt = new Uint8Array(100)
    let ct_1 = rs_prp.forward(pt)
    let ct_2 = ts_prp.permutate(pt)

    console.log(u8aToHex(ct_1))
    console.log(u8aToHex(ct_2))
    assert.equal(u8aToHex(ct_1), u8aToHex(ct_2), 'ciphertexts dont correspond')
  })

  it('PRP correspondence - TS forward / RS inverse', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    let rs_prp_params = new Rust_PRPParameters(secret)
    let ts_prp_params = derivePRPParameters(secret)

    assert.equal(u8aToHex(ts_prp_params.key), u8aToHex(rs_prp_params.key()), 'keys dont correspond')
    assert.equal(u8aToHex(ts_prp_params.iv), u8aToHex(rs_prp_params.iv()), 'ivs dont correspond')

    let rs_prp = new Rust_PRP(rs_prp_params)
    let ts_prp = TS_PRP.createPRP(ts_prp_params)

    let pt_1 = new Uint8Array(100)
    let ct = ts_prp.permutate(pt_1)
    let pt_2 = rs_prp.inverse(ct)

    assert.equal(u8aToHex(new Uint8Array(100)), u8aToHex(pt_2), 'plaintexts dont correspond')
  })

  it('PRP correspondence - RS forward / TS inverse', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    let rs_prp_params = new Rust_PRPParameters(secret)
    let ts_prp_params = derivePRPParameters(secret)

    assert.equal(u8aToHex(ts_prp_params.key), u8aToHex(rs_prp_params.key()), 'keys dont correspond')
    assert.equal(u8aToHex(ts_prp_params.iv), u8aToHex(rs_prp_params.iv()), 'ivs dont correspond')

    let rs_prp = new Rust_PRP(rs_prp_params)
    let ts_prp = TS_PRP.createPRP(ts_prp_params)

    let pt_1 = new Uint8Array(100)
    let ct = rs_prp.forward(pt_1)
    let pt_2 = ts_prp.inverse(ct)

    assert.equal(u8aToHex(pt_1), u8aToHex(pt_2), 'plaintexts dont correspond')
  })

  it('keyshares correspondence: generate key shares in RS and verify them in TS', async function () {
    const AMOUNT = 4
    const peerIds = await Promise.all(Array.from({ length: AMOUNT }).map((_) => createSecp256k1PeerId()))

    let peer_pub_keys = peerIds.map((p) => (p.publicKey as Uint8Array).slice(4))
    let keyshares_rs = SharedKeys.generate(peer_pub_keys)

    assert.equal(keyshares_rs.count_shared_keys(), AMOUNT)

    const alpha = keyshares_rs.get_alpha()
    const secrets = Array.from({ length: keyshares_rs.count_shared_keys() }).map((_, i) =>
      keyshares_rs.get_peer_shared_key(i)
    )

    for (let i = 0; i < AMOUNT; i++) {
      const { alpha: tmpAlpha, secret } = forwardTransform(alpha, peerIds[i])

      assert(u8aEquals(secret, secrets[i]))

      alpha.set(tmpAlpha)
    }
  })

  it('keyshares correspondence: generate key shares in TS and verify them in RS', async function () {
    const AMOUNT = 4
    const keyPairs = await Promise.all(Array.from({ length: AMOUNT }).map((_) => createSecp256k1PeerId()))

    const { alpha, secrets } = generateKeyShares(keyPairs)

    for (let i = 0; i < AMOUNT; i++) {
      let fwd_keyshare_rs = SharedKeys.forward_transform(
        alpha,
        keyPairs[i].publicKey.slice(4),
        keyPairs[i].privateKey.slice(4)
      )

      assert.equal(fwd_keyshare_rs.count_shared_keys(), 1)

      const tmpAlpha = fwd_keyshare_rs.get_alpha()
      const secret = fwd_keyshare_rs.get_peer_shared_key(0)

      assert(u8aEquals(secret, secrets[i]))

      alpha.set(tmpAlpha)
    }
  })
})
