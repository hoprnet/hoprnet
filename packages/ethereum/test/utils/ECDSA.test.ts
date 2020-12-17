import { deployments } from 'hardhat'
import { constants } from '@openzeppelin/test-helpers'
import Web3 from 'web3'
import { signMessage, prefixMessage } from '../utils'
import { ACCOUNT_A } from '../constants'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'

const { soliditySha3 } = Web3.utils

const ECDSAMock = artifacts.require('ECDSAMock')

const useFixtures = deployments.createFixture(async () => {
  const ECDSA = await ECDSAMock.new()

  return {
    ECDSA
  }
})

describe('ECDSA', function () {
  it('should convert public key to address', async function () {
    const { ECDSA } = await useFixtures()

    const address = await ECDSA.pubKeyToEthereumAddress(ACCOUNT_A.pubKeyFirstHalf, ACCOUNT_A.pubKeySecondHalf)

    expect(address).to.equal(ACCOUNT_A.address)
  })

  it('should validate public key', async function () {
    const { ECDSA } = await useFixtures()

    const valid = await ECDSA.validate(ACCOUNT_A.pubKeyFirstHalf, ACCOUNT_A.pubKeySecondHalf)

    expect(valid).to.be.true
  })

  it('should fail to validate public key', async function () {
    const { ECDSA } = await useFixtures()

    const valid = await ECDSA.validate(ACCOUNT_A.pubKeyFirstHalf, constants.ZERO_BYTES32)

    expect(valid).to.be.false
  })

  // @TODO: add more recover tests
  it('should recover signer', async function () {
    const { ECDSA } = await useFixtures()
    const message = soliditySha3({
      type: 'string',
      value: 'hello world'
    })
    const { r, s, v } = signMessage(message, stringToU8a(ACCOUNT_A.privKey))

    // why add 27? https://bitcoin.stackexchange.com/a/38909
    const signer = await ECDSA.recover(message, u8aToHex(r), u8aToHex(s), v + 27)

    expect(signer).to.equal(ACCOUNT_A.address)
  })

  it('should prefix and hash message', async function () {
    const { ECDSA } = await useFixtures()
    const message = soliditySha3({
      type: 'string',
      value: 'hello world'
    })
    const prefixed = u8aToHex(prefixMessage(message))

    const result = await ECDSA.toEthSignedMessageHash('39', message)

    expect(prefixed).to.equal(result)
  })
})
