import assert from 'assert'
import rewiremock from 'rewiremock'
import sinon from 'sinon'

let getBootstrapAddresses: any = null
//let mockPromises = sinon.fake() as any

const mockMultiAddrss = '/ip4/34.65.75.45/tcp/9091/p2p/16Uiu2HAm2cjqsDMmprtN2QKaq3LJrq3YK7vtdbQQFsxGrhLRoYsy'
const resolveTxt = sinon.fake.returns([mockMultiAddrss])

describe('getBootstrapAddresses', function () {
  beforeEach(async () => {
    rewiremock('dns').with({
      promises: { resolveTxt }
    })
    rewiremock.enable()
    const BootstrapUtil = await import('./bootstrap')
    getBootstrapAddresses = BootstrapUtil.getBootstrapAddresses
  })

  it('passed addresses resolved first', async function () {
    assert(await getBootstrapAddresses(mockMultiAddrss))
    assert(!resolveTxt.calledOnce)
  })
  it('falls back to dns', async function () {
    assert(await getBootstrapAddresses())
    assert(resolveTxt.calledOnce)
  })

  afterEach(() => rewiremock.disable())
})
