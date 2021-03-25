import assert from 'assert'
import {
  Header,
  deriveBlinding,
  deriveCipherParameters,
  deriveTagParameters,
  createMAC,
  deriveTicketKey,
  derivePRGParameters
} from '.'
import { Utils } from '@hoprnet/hopr-core-ethereum'
import PeerId from 'peer-id'
import Hopr from '../../../'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { randomBytes } from 'crypto'
import secp256k1 from 'secp256k1'
import { u8aEquals } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '../../../constants'

if (MAX_HOPS > 1) {
  describe('test creation & transformation of a header', function () {
    async function createAndDecomposeHeader(
      node: Hopr<HoprCoreConnector>,
      peerIds: PeerId[]
    ): Promise<{ header: Header; identifier: Uint8Array; secrets: Uint8Array[] }> {
      const { header, identifier, secrets } = await Header.create<HoprCoreConnector>(node, peerIds)

      for (let i = 0; i < peerIds.length - 1; i++) {
        header.deriveSecret(peerIds[i].privKey.marshal())
        assert(u8aEquals(header.derivedSecret, secrets[i]), `pre-computed secret and derived secret should be the same`)
        assert(header.verify(), `MAC must be valid`)

        header.extractHeaderInformation()
        assert(
          peerIds[i + 1].pubKey.marshal().every((value: number, index: number) => value == header.address[index]),
          `Decrypted address should be the same as the one of node ${i + 1}`
        )
        header.transformForNextNode()
      }

      return { header, identifier, secrets }
    }

    function getNode(): Hopr<HoprCoreConnector> {
      const node = ({
        paymentChannels: {
          utils: Utils
        }
      } as unknown) as Hopr<HoprCoreConnector>

      return node
    }

    it('should derive parameters', function () {
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
        notEqualHelper([
          blinding,
          encryptionKey.iv,
          encryptionKey.key,
          tagParameter,
          mac,
          transactionKey,
          prgParameters.key,
          prgParameters.iv
        ]),
        'Keys should all be with high probability different'
      )
    })

    it('should create a header', async function () {
      const peerIds = await Promise.all(
        Array.from({
          length: MAX_HOPS
        }).map(() => PeerId.create({ keyType: 'secp256k1' }))
      )

      const { header, identifier, secrets } = await createAndDecomposeHeader(getNode(), peerIds)
      const lastPeerId = peerIds[peerIds.length - 1]
      const lastSecret = secrets[secrets.length - 1]

      header.deriveSecret(lastPeerId.privKey.marshal(), true)
      assert(u8aEquals(header.derivedSecret, lastSecret), `pre-computed secret and derived secret should be the same`)

      assert(header.verify(), `MAC should be valid`)
      header.extractHeaderInformation(true)

      assert(
        u8aEquals(lastPeerId.pubKey.marshal(), header.address),
        `Decrypted address should be the same as the final recipient`
      )

      assert(u8aEquals(header.identifier, identifier), `Decrypted identifier should have the expected value`)
    })

    it('should create a header with a path less than MAX_HOPS nodes', async function () {
      const peerIds = await Promise.all([
        PeerId.create({ keyType: 'secp256k1' }),
        PeerId.create({ keyType: 'secp256k1' })
      ])

      const { header, identifier, secrets } = await createAndDecomposeHeader(getNode(), peerIds)

      header.deriveSecret(peerIds[1].privKey.marshal(), true)
      assert(u8aEquals(header.derivedSecret, secrets[1]), `pre-computed secret and derived secret should be the same`)

      assert(header.verify(), `MAC must be valid`)
      header.extractHeaderInformation(true)

      assert(
        u8aEquals(peerIds[1].pubKey.marshal(), header.address),
        `Decrypted address should be the same as the final recipient`
      )

      assert(u8aEquals(header.identifier, identifier), `Decrypted identifier should have the expected value`)
    })

    it('should create a header with exactly two nodes', async function () {
      const peerIds = [await PeerId.create({ keyType: 'secp256k1' })]

      const { header, identifier, secrets } = await createAndDecomposeHeader(getNode(), peerIds)

      header.deriveSecret(peerIds[0].privKey.marshal(), true)
      assert(u8aEquals(header.derivedSecret, secrets[0]), `pre-computed secret and derived secret should be the same`)

      assert(header.verify(), `MAC must be valid`)
      header.extractHeaderInformation(true)

      assert(
        u8aEquals(peerIds[0].pubKey.marshal(), header.address),
        `Decrypted address should be the same as the final recipient`
      )

      assert(u8aEquals(header.identifier, identifier), `Decrypted identifier should have the expected value`)
    })
  })

  function notEqualHelper(arr: Uint8Array[]) {
    for (let i = 0; i < arr.length; i++) {
      for (let j = i + 1; j < arr.length; j++) {
        if (arr[i].length == arr[j].length && u8aEquals(arr[i], arr[j])) {
          return false
        }
      }
    }
    return true
  }
}
