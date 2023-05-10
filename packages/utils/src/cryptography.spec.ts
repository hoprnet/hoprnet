import {
  Hash as Rust_HASH,
  PRG as Rust_PRG,
  PRGParameters as Rust_PRGParameters,
  PRP as Rust_PRP,
  PRPParameters as Rust_PRPParameters,
  SharedKeys,
  derive_packet_tag,
  iterate_hash,
  recover_iterated_hash,
  create_tagged_mac,
  PublicKey,
  Signature
} from '../../core/lib/core_crypto.js'

import {
  generateKeyShares,
  iterateHash,
  PRG as TS_PRG,
  PRP as TS_PRP,
  recoverIteratedHash,
  createMAC,
  keyShareTransform as forwardTransform,
  derivePacketTag,
  derivePRGParameters,
  derivePRPParameters,
} from './crypto/index.js'

import { Hash } from './types/index.js'

import {PublicKey as TsPublicKey, Signature as TsSignature} from './types/index.js'

import assert from 'assert'

import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import { stringToU8a, u8aEquals, u8aToHex } from './u8a/index.js'
import { SECRET_LENGTH } from './constants.js'

describe('cryptographic correspondence tests', async function () {
  it('digest correspondence', async function () {
    let data = new Uint8Array(32).fill(1)
    let h1 = Hash.create(data).serialize()
    let h2 = Rust_HASH.create([data]).serialize()
    assert(u8aEquals(h1, h2))
  })

  it('mac correspondence', async function () {
    let data = new Uint8Array(32).fill(1)
    let key = new Uint8Array(32)
    let m1 = create_tagged_mac(key, data)
    let m2 = createMAC(key, data)
    assert(u8aEquals(m1, m2))
  })

  it('derived parameters correspondence', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    let packet_tag_ts = derivePacketTag(secret)
    let packet_tag_rs = derive_packet_tag(secret)
    assert.equal(u8aToHex(packet_tag_ts), u8aToHex(packet_tag_rs))
  })

  it('PRG correspondence', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = Rust_PRG.from_parameters(rs_prg_params).digest(5, 10)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(5, 10)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = Rust_PRG.from_parameters(rs_prg_params).digest(0, 100)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(0, 100)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = Rust_PRG.from_parameters(rs_prg_params).digest(10, 100)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(10, 100)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }

    {
      let rs_prg_params = new Rust_PRGParameters(secret)
      let rs_output = Rust_PRG.from_parameters(rs_prg_params).digest(16, 22)

      let ts_prg_params = derivePRGParameters(secret)
      let ts_output = TS_PRG.createPRG(ts_prg_params).digest(16, 22)

      assert.equal(u8aToHex(rs_output), u8aToHex(ts_output))
    }
  })

  it('PRP correspondence - same ciphertext', async function () {
    let secret = new Uint8Array(SECRET_LENGTH)

    let rs_prp_params = new Rust_PRPParameters(secret)
    let ts_prp_params = derivePRPParameters(secret)

    assert.equal(u8aToHex(ts_prp_params.key), u8aToHex(rs_prp_params.key()), 'keys dont correspond')
    assert.equal(u8aToHex(ts_prp_params.iv), u8aToHex(rs_prp_params.iv()), 'ivs dont correspond')

    let ts_prp = TS_PRP.createPRP(ts_prp_params)
    let rs_prp = new Rust_PRP(rs_prp_params)

    let pt = new Uint8Array(100)
    let ct_1 = rs_prp.forward(pt)
    let ct_2 = ts_prp.permutate(pt)

    assert.equal(u8aToHex(ct_1), u8aToHex(ct_2))
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

  it('keyshares correspondence generate key shares in RS and verify them in TS', async function () {
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

  it('secp256k1 public key correspondence', async function() {
    let ts_pub = TsPublicKey.fromPrivKeyString('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775')
    let rs_pub = PublicKey.from_privkey(stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775'))
    assert(u8aEquals(ts_pub.serializeCompressed(), rs_pub.serialize(true)))
    assert(u8aEquals(ts_pub.serializeUncompressed(), rs_pub.serialize(false)))
    assert.equal(
      '0x' + PublicKey.deserialize(ts_pub.serializeCompressed()).to_hex(true),
      TsPublicKey.deserialize(rs_pub.serialize(true)).toCompressedPubKeyHex()
    )
    assert.equal(
      '0x' + PublicKey.deserialize(ts_pub.serializeUncompressed()).to_hex(false),
      TsPublicKey.deserialize(rs_pub.serialize(false)).toUncompressedPubKeyHex()
    )
  })

  it('signature correspondence tests', async function() {
    let priv_key = stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775')
    let rs_pub = PublicKey.from_privkey(priv_key)
    let message = Hash.create(stringToU8a('0xdeadbeefcafebabe')).serialize()

    let rs_sgn = Signature.sign_hash(message, priv_key)
    let ts_sgn = TsSignature.deserialize(rs_sgn.serialize())
    assert(ts_sgn.verify(message, TsPublicKey.fromPeerIdString(rs_pub.to_peerid_str())))

    let rs_sgn_2 = TsSignature.create(message, priv_key)
    assert(Signature.deserialize(rs_sgn_2.serialize()).verify_hash_with_pubkey(message, rs_pub))
  })

  it('keyshares correspondence generate key shares in TS and verify them in RS', async function () {
    const AMOUNT = 4
    const keyPairs = await Promise.all(Array.from({ length: AMOUNT }).map((_) => createSecp256k1PeerId()))

    const { alpha, secrets } = generateKeyShares(keyPairs)

    for (let i = 0; i < AMOUNT; i++) {
      let fwd_keyshare_rs = SharedKeys.forward_transform(alpha, keyPairs[i].privateKey.slice(4))

      assert.equal(fwd_keyshare_rs.count_shared_keys(), 1)

      const tmpAlpha = fwd_keyshare_rs.get_alpha()
      const secret = fwd_keyshare_rs.get_peer_shared_key(0)

      assert(u8aEquals(secret, secrets[i]))

      alpha.set(tmpAlpha)
    }
  })

  it('correspondence of iterated hash & recovery', async function () {
    let seed = new Uint8Array(16)
    let hashFn = (msg: Uint8Array) => Hash.create(msg).serialize().slice(0, Hash.SIZE)
    let TS_iterated = await iterateHash(seed, hashFn, 1000, 10)
    let RS_iterated = iterate_hash(seed, 1000, 10)

    assert(u8aEquals(TS_iterated.hash, RS_iterated.hash()))
    assert.equal(TS_iterated.intermediates.length, RS_iterated.count_intermediates())

    for (let i = 0; i < RS_iterated.count_intermediates(); i++) {
      assert.equal(TS_iterated.intermediates[i].iteration, RS_iterated.intermediate(i).iteration)
      assert(u8aEquals(TS_iterated.intermediates[i].preImage, RS_iterated.intermediate(i).intermediate))
    }

    let RS_hint = RS_iterated.intermediate(98)
    assert.equal(RS_hint.iteration, 980)
    assert(
      u8aEquals(RS_hint.intermediate, stringToU8a('a380d145d8612d33912494f1b36571c0b59b9bd459e6bb7d5ea05946be4c256b'))
    )

    let target_idx = 988
    let target_hash = stringToU8a('614eeebc22e8a79cbcac8bb6ba140768dd4bee4017460ad941de72f0fd5610e3')

    let TS_recovered = await recoverIteratedHash(
      target_hash,
      hashFn,
      async (i) => (i == RS_hint.iteration ? RS_hint.intermediate : undefined),
      1000,
      10,
      undefined
    )
    assert(TS_recovered != undefined)

    let TS_hint = TS_iterated.intermediates[98]
    assert.equal(TS_hint.iteration, 980)
    let RS_recovered = recover_iterated_hash(
      target_hash,
      (i: number) => (i == TS_hint.iteration ? TS_hint.preImage : undefined),
      1000,
      10,
      undefined
    )
    assert(RS_recovered != undefined)

    assert.equal(TS_recovered.iteration, RS_recovered.iteration)
    assert(u8aEquals(TS_recovered.preImage, RS_recovered.intermediate))
    assert.equal(target_idx, RS_recovered.iteration)
  })
})
