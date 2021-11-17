import assert from 'assert'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { utils } from 'ethers'
import { UnreleasedTokens } from './utils'

const unreleasedTokenInput: UnreleasedTokens = require('./unreleasedTokens.json')
const unreleasedTokenInputs: UnreleasedTokens[] = [
  { ...unreleasedTokenInput },
  { ...unreleasedTokenInput, link: {} },
  {
    ...unreleasedTokenInput,
    link: {
      '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1': [
        '0x0051437667689b36f9cfec31e4f007f1497c0f98',
        '0x0ddae82c2080aae8e57814e5636dbbfa775be929'
      ],
      '16Uiu2HAmBCcc822eURPRu6YXuSNmPZn2tJ1nEePNPUsz8xRNZRV7': ['0x1d216b8706be76f7906eb5872835ce5567fd2ef5']
    }
  }
]

unreleasedTokenInputs.forEach(function (input, index) {
  describe('test unreleasedTokens.json of case' + index, function () {
    let unreleasedTokens: UnreleasedTokens

    beforeEach(async () => {
      unreleasedTokens = input
    })

    it('should validate allocation entries', function () {
      for (const nodeEthAddress of Object.keys(unreleasedTokens.allocation)) {
        assert(utils.isAddress(nodeEthAddress), `invalid ethAddress: ${nodeEthAddress}`)

        // order by lower block, ascending
        const allocations = unreleasedTokens.allocation[nodeEthAddress].sort((a, b) => a.lowerBlock - b.lowerBlock)
        for (let index = 0; index < allocations.length; index++) {
          const allocation = allocations[index]
          assert(new BN(allocation.unreleased).gtn(0), `invalid unreleased tokens: ${allocation.unreleased}`)
          assert(
            new BN(allocation.lowerBlock).ltn(allocation.upperBlock),
            `invalid unreleased tokens: ${allocation.unreleased}`
          )
          if (index > 0) {
            assert(
              new BN(allocation.lowerBlock).gten(allocations[index - 1].upperBlock),
              `overlapping schedule: ${allocation.unreleased}`
            )
          }
        }
      }
    })

    it('should validate link entries', function () {
      for (const hoprId of Object.keys(unreleasedTokens.link)) {
        let validHoprId = false
        let errMsg = ''
        try {
          PeerId.createFromB58String(hoprId)
          validHoprId = true
        } catch (err) {
          errMsg = err
        }
        assert(validHoprId, `invalid hoprId: ${hoprId}, ${errMsg}`)
      }
    })

    it('should have no duplicate entries in allocation', function () {
      const nodeAddresses = Object.keys(unreleasedTokens.allocation)
      for (const nodeAddress of nodeAddresses) {
        const count = nodeAddresses.filter((adr) => adr === nodeAddress).length
        assert(count === 1, `ethAddress ${nodeAddress} exists more than once`)
      }
    })

    it('should only have known allocation address in link', function () {
      const allocationAddresses = Object.keys(unreleasedTokens.allocation)
      for (const nodeAddresses of Object.values(unreleasedTokens.link)) {
        for (const nodeAddress of nodeAddresses) {
          const index = allocationAddresses.findIndex((adr) => adr === nodeAddress)
          assert(index >= 0, `ethAddress ${nodeAddress} does not exist in allocation`)
        }
      }
    })
  })
})
