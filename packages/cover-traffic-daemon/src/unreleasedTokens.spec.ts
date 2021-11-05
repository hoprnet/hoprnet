import assert from 'assert'
import BigNumber from 'bignumber.js'
import PeerId from 'peer-id'
import { utils } from 'ethers'

describe('test unreleasedTokens.json', function () {
  const unreleasedTokens: {
    ethAddress: string
    hoprId: string
    tokens: string
  }[] = require('./unreleasedTokens.json')

  it('should validate entries', function () {
    for (const { ethAddress, hoprId, tokens } of unreleasedTokens) {
      assert(utils.isAddress(ethAddress), `invalid ethAddress: ${ethAddress}`)

      let validHoprId = false
      let errMsg = ''
      try {
        PeerId.createFromB58String(hoprId)
        validHoprId = true
      } catch (err) {
        errMsg = err
      }
      assert(validHoprId, `invalid hoprId: ${hoprId}, ${errMsg}`)

      assert(new BigNumber(tokens).gt(0), `invalid tokens: ${tokens}`)
    }
  })

  it('should have no duplicate entries', function () {
    for (const { ethAddress } of unreleasedTokens) {
      const count = unreleasedTokens.filter(
        (o) => utils.getAddress(o.ethAddress) === utils.getAddress(ethAddress)
      ).length
      assert(count === 1, `ethAddress ${ethAddress} exists more than once`)
    }
  })
})
