import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { signMessage, prefixMessageWithHOPR } from '../utils'
import { ACCOUNT_A } from '../constants'
import { ECDSAMock__factory } from '../../types'

const { solidityKeccak256 } = ethers.utils

const useFixtures = deployments.createFixture(async () => {
  const [signer] = await ethers.getSigners()
  const ECDSA = await new ECDSAMock__factory(signer).deploy()

  return {
    ECDSA
  }
})

describe.only('ECDSA', function () {
  it('should convert uncompressed public key to address', async function () {
    const { ECDSA } = await useFixtures()

    const address = await ECDSA.uncompressedPubKeyToAddress(ACCOUNT_A.uncompressedPublicKey)

    expect(address).to.equal(ACCOUNT_A.address)
  })

  // @TODO: add more recover tests
  it('should recover signer', async function () {
    const { ECDSA } = await useFixtures()
    const message = solidityKeccak256(['string'], ['hello world'])
    const { r, s, v } = await signMessage(message, ACCOUNT_A.privateKey)

    const signer = await ECDSA.recover(message, r, s, v)

    expect(signer).to.equal(ACCOUNT_A.address)
  })

  it('should prefix and hash message', async function () {
    const { ECDSA } = await useFixtures()
    const message = solidityKeccak256(['string'], ['hello world'])
    const prefixed = prefixMessageWithHOPR(message)

    const result = await ECDSA.toEthSignedMessageHash('39', message)

    expect(prefixed).to.equal(result)
  })
})
