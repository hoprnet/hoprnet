import { durations } from '@hoprnet/hopr-utils'
import { compile } from '@hoprnet/hopr-ethereum'

before(async function () {
  this.timeout(durations.minutes(30))
  await compile()
})
