import assert from 'assert'
import { getBootstrapAddresses } from './bootstrap'

describe('getBootstrapAddresses', function () {
  it('passed addresses resolved first', async function () {
    assert(
      await getBootstrapAddresses('/ip4/34.65.75.45/tcp/9091/p2p/16Uiu2HAm2cjqsDMmprtN2QKaq3LJrq3YK7vtdbQQFsxGrhLRoYsy')
    )
  })
  it('falls back to dns', async function () {
    assert(await getBootstrapAddresses())
  })
})
