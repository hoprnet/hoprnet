import { createRoutingInfo, forwardTransform, type RelayNodeOutput } from './routingInfo.js'
import { generateKeyShares } from './keyShares.js'
import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'
import { u8aEquals } from '../../u8a/index.js'
import assert from 'assert'

describe('routing info generation and mutation', function () {
  it('generate routing info and transform it', async function () {
    const AMOUNT = 3

    const peerIds = await Promise.all(Array.from({ length: AMOUNT }, (_) => createSecp256k1PeerId()))

    const { secrets } = generateKeyShares(peerIds)
    const maxHops = 3

    const { routingInformation, mac } = createRoutingInfo(
      maxHops,
      peerIds,
      secrets,
      0,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    let transformedOutput: ReturnType<typeof forwardTransform>

    for (let i = 0; i < secrets.length; i++) {
      transformedOutput = forwardTransform(
        secrets[i],
        routingInformation,
        (transformedOutput as RelayNodeOutput)?.mac ?? mac,
        maxHops,
        0,
        0
      )

      if (i < secrets.length - 1) {
        assert(transformedOutput.lastNode == false)

        assert(u8aEquals(unmarshalPublicKey(peerIds[i + 1].publicKey).marshal(), transformedOutput.nextNode))
      } else {
        assert(transformedOutput.lastNode == true)

        assert(transformedOutput.additionalData.length == 0)
      }
    }
  })

  it('generate routing info and transform it - reduced path', async function () {
    const AMOUNT = 2

    const peerIds = await Promise.all(Array.from({ length: AMOUNT }, (_) => createSecp256k1PeerId()))

    const { secrets } = generateKeyShares(peerIds)
    const maxHops = 3

    const { routingInformation, mac } = createRoutingInfo(
      maxHops,
      peerIds,
      secrets,
      0,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    let transformedOutput: ReturnType<typeof forwardTransform>
    for (let i = 0; i < secrets.length; i++) {
      transformedOutput = forwardTransform(
        secrets[i],
        routingInformation,
        (transformedOutput as RelayNodeOutput)?.mac ?? mac,
        maxHops,
        0,
        0
      )

      if (i < secrets.length - 1) {
        assert(transformedOutput.lastNode == false)

        assert(u8aEquals(unmarshalPublicKey(peerIds[i + 1].publicKey).marshal(), transformedOutput.nextNode))
      } else {
        assert(transformedOutput.lastNode == true)

        assert(transformedOutput.additionalData.length == 0)
      }
    }
  })

  it('generate routing info and transform it - zero hop (no filler)', async function () {
    const AMOUNT = 1

    const peerIds = await Promise.all(Array.from({ length: AMOUNT }, (_) => createSecp256k1PeerId()))

    const { secrets } = generateKeyShares(peerIds)
    const maxHops = 3

    const { routingInformation, mac } = createRoutingInfo(
      maxHops,
      peerIds,
      secrets,
      0,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    let transformedOutput: ReturnType<typeof forwardTransform>

    for (let i = 0; i < secrets.length; i++) {
      transformedOutput = forwardTransform(
        secrets[i],
        routingInformation,
        (transformedOutput as RelayNodeOutput)?.mac ?? mac,
        maxHops,
        0,
        0
      )

      if (i < secrets.length - 1) {
        assert(transformedOutput.lastNode == false)

        assert(
          u8aEquals(
            unmarshalPublicKey(peerIds[i + 1].publicKey).marshal(),
            (transformedOutput as RelayNodeOutput).nextNode
          )
        )
      } else {
        assert(transformedOutput.lastNode == true)

        assert(transformedOutput.additionalData.length == 0)
      }
    }
  })
})
