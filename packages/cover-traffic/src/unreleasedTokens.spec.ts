import assert from 'assert'
import BigNumber from 'bignumber.js'
import PeerId from 'peer-id'
import { utils } from 'ethers'
import { stringToU8a } from '@hoprnet/hopr-utils'
import UNRELEASED_TOKENS from './unreleasedTokens.json'

describe('test unreleasedTokens.json', function () {
  it('should validate entries', function () {
    for (const { tokens, ethAddress, hoprId } of UNRELEASED_TOKENS as {
      tokens: string
      ethAddress: string
      hoprId: string
    }[]) {
      assert(new BigNumber(tokens).lt(0), `invalid tokens in entry for ${ethAddress}`)
      assert(utils.isAddress(ethAddress), `invalid ethAddress in entry for ${ethAddress}`)
      assert(new PeerId(stringToU8a(hoprId)).isValid(), `invalid hoprId in entry for ${ethAddress}`)
    }
  })
})
