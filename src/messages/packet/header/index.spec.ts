import assert from 'assert'
import { Header, deriveBlinding, deriveCipherParameters, deriveTagParameters, createMAC, deriveTicketKey, derivePRGParameters } from '.'
import { Utils, Constants } from '@hoprnet/hopr-core-polkadot'
import PeerId from 'peer-id'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import { randomBytes } from 'crypto'
import secp256k1 from 'secp256k1'

describe('test creation & transformation of a header', function() {
  function createAndDecomposeHeader(peerIds: PeerId[]): { header: Header<HoprCoreConnectorInstance>, identifier: Uint8Array, secrets: Uint8Array[] } {
    const { header, identifier, secrets } = Header.create<HoprCoreConnectorInstance>(peerIds)

    for (let i = 0; i < peerIds.length - 1; i++) {
      header.deriveSecret(peerIds[i].privKey.marshal())
      assert.deepEqual(header.derivedSecret, secrets[i], `pre-computed secret and derived secret should be the same`)
      assert(header.verify(), `MAC must be valid`)

      header.extractHeaderInformation()
      assert(
        peerIds[i + 1].pubKey.marshal().every((value: number, index: number) => value == header.address[index]),
        `Decrypted address should be the same as the of node ${i + 1}`
      )
      header.transformForNextNode()
    }

    return { header, identifier, secrets}
  }

  it('should derive parameters', function() {
    const secret = randomBytes(32)
    const alpha = randomBytes(32)

    const secretGroupElement = secp256k1.publicKeyCreate(secret)

    const blinding = deriveBlinding(secp256k1.publicKeyCreate(alpha), secretGroupElement)
    const encryptionKey = deriveCipherParameters(secretGroupElement)
    const tagParameter = deriveTagParameters(secretGroupElement)
    const mac = createMAC(secretGroupElement, new TextEncoder().encode('test'))
    const transactionKey = deriveTicketKey(secretGroupElement)
    const prgParameters = derivePRGParameters(secretGroupElement)

    assert(
      notEqualHelper([blinding, encryptionKey.iv, encryptionKey.key, tagParameter, mac, transactionKey, prgParameters.key, prgParameters.iv]),
      'Keys should all be with high probability different'
    )
  })

  it('should create a header', async function() {
    const peerIds = await Promise.all([
      PeerId.create({ keyType: 'secp256k1' }),
      PeerId.create({ keyType: 'secp256k1' }),
      PeerId.create({ keyType: 'secp256k1' })
    ])

    const { header, identifier, secrets } = createAndDecomposeHeader(peerIds)

    header.deriveSecret(peerIds[2].privKey.marshal(), true)
    assert.deepEqual(header.derivedSecret, secrets[2], `pre-computed secret and derived secret should be the same`)

    assert(header.verify(), `MAC should be valid`)
    header.extractHeaderInformation(true)

    assert(
      peerIds[2].pubKey.marshal().every((value: number, index: number) => value == header.address[index]),
      `Decrypted address should be the same as the final recipient`
    )

    assert(
      header.identifier.every((value: number, index: number) => value == identifier[index]),
      `Decrypted identifier should have the expected value`
    )
  })

  it('should create a header with a path less than MAX_HOPS nodes', async function () {
    const peerIds = await Promise.all([
      PeerId.create({ keyType: 'secp256k1' }),
      PeerId.create({ keyType: 'secp256k1' }),
    ])

    const { header, identifier, secrets } = createAndDecomposeHeader(peerIds)

    header.deriveSecret(peerIds[1].privKey.marshal(), true)
    assert.deepEqual(header.derivedSecret, secrets[1], `pre-computed secret and derived secret should be the same`)

    assert(header.verify(), `MAC must be valid`)
    header.extractHeaderInformation(true)


    assert(
      peerIds[1].pubKey.marshal().every((value: number, index: number) => value == header.address[index]),
      `Decrypted address should be the same as the final recipient`
    )

    assert(
      header.identifier.every((value: number, index: number) => value == identifier[index]),
      `Decrypted identifier should have the expected value`
    )
  })



  it('should create a header with exactly two nodes', async function () {
    const peerIds = await Promise.all([
      PeerId.create({ keyType: 'secp256k1' }),
    ])

    const { header, identifier, secrets } = createAndDecomposeHeader(peerIds)

    header.deriveSecret(peerIds[0].privKey.marshal(), true)
    assert.deepEqual(header.derivedSecret, secrets[0], `pre-computed secret and derived secret should be the same`)

    assert(header.verify(), `MAC must be valid`)
    header.extractHeaderInformation(true)


    assert(
      peerIds[0].pubKey.marshal().every((value: number, index: number) => value == header.address[index]),
      `Decrypted address should be the same as the final recipient`
    )

    assert(
      header.identifier.every((value: number, index: number) => value == identifier[index]),
      `Decrypted identifier should have the expected value`
    )
  })
})

function notEqualHelper(arr: Uint8Array[]) {
  for (let i = 0; i < arr.length; i++) {
    for (let j = i + 1; j < arr.length; j++) {
      if (arr[i].length == arr[j].length && arr[i].every((value: number, index: number) => arr[j][index] == value)) {
        return false
      }
    }
  }
  return true
}
