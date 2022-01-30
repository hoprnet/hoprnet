import assert from 'assert'
import sinon from 'sinon'
import { getAddresses } from './address'

let node = sinon.fake() as any

describe('test address', function () {
  it('should get addresses', async function () {
    node.getEthereumAddress = sinon.fake.returns('0xEA9eDAE5CfC794B75C45c8fa89b605508A03742a')
    node.getId = sinon.fake.returns('16Uiu2HAmVfV4GKQhdECMqYmUMGLy84RjTJQxTWDcmUX5847roBar')

    const { native, hopr } = getAddresses(node)
    assert.equal(native, '0xEA9eDAE5CfC794B75C45c8fa89b605508A03742a')
    assert.equal(hopr, '16Uiu2HAmVfV4GKQhdECMqYmUMGLy84RjTJQxTWDcmUX5847roBar')
  })
})
