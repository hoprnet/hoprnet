import { createRoutingInfo, forwardTransform } from './routingInfo'
import { generateKeyShares } from './keyShares'
import PeerId from 'peer-id'
import { u8aEquals } from '../../u8a'
import assert from 'assert'

describe('routing info generation and mutation', function () {
  it('generate routing info and transform it', async function () {
    const AMOUNT = 3

    const peerIds = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const [_, secrets] = generateKeyShares(peerIds)
    const maxHops = 3

    const [header, mac] = createRoutingInfo(
      maxHops,
      peerIds,
      secrets,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    let transformedOutput: Uint8Array[]
    for (let i = 0; i < secrets.length; i++) {
      transformedOutput = forwardTransform(secrets[i], header, transformedOutput?.[1] ?? mac, maxHops, 0, 0)

      if (i < secrets.length - 1) {
        assert(transformedOutput != undefined)

        assert(u8aEquals(peerIds[i + 1].pubKey.marshal(), transformedOutput[2]))
      } else {
        assert(transformedOutput == undefined)
      }
    }
  })

  it('generate routing info and transform it - reduced path', async function () {
    const AMOUNT = 2

    const peerIds = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const [_, secrets] = generateKeyShares(peerIds)
    const maxHops = 3

    const [header, mac] = createRoutingInfo(
      maxHops,
      peerIds,
      secrets,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    let transformedOutput: Uint8Array[]
    for (let i = 0; i < secrets.length; i++) {
      transformedOutput = forwardTransform(secrets[i], header, transformedOutput?.[1] ?? mac, maxHops, 0, 0)

      if (i < secrets.length - 1) {
        assert(transformedOutput != undefined)

        assert(u8aEquals(peerIds[i + 1].pubKey.marshal(), transformedOutput[2]))
      } else {
        assert(transformedOutput == undefined)
      }
    }
  })

  it('generate routing info and transform it - zero hop (no filler)', async function () {
    const AMOUNT = 1

    const peerIds = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const [_, secrets] = generateKeyShares(peerIds)
    const maxHops = 3

    const [header, mac] = createRoutingInfo(
      maxHops,
      peerIds,
      secrets,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    let transformedOutput: Uint8Array[]
    for (let i = 0; i < secrets.length; i++) {
      transformedOutput = forwardTransform(secrets[i], header, transformedOutput?.[1] ?? mac, maxHops, 0, 0)

      if (i < secrets.length - 1) {
        assert(transformedOutput != undefined)

        assert(u8aEquals(peerIds[i + 1].pubKey.marshal(), transformedOutput[2]))
      } else {
        assert(transformedOutput == undefined)
      }
    }
  })
})
