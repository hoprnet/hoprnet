import assert from 'assert'
import sinon from 'sinon'
import PeerId from 'peer-id'
import { Address } from '@hoprnet/hopr-utils'
import { getAddresses } from './address'

let node = sinon.fake() as any

describe('test address', function () {
  it('should get addresses', async function () {
    node.getEthereumAddress = sinon.fake.returns(Address.fromString('0xEA9eDAE5CfC794B75C45c8fa89b605508A03742a'))
    node.getId = sinon.fake.returns(PeerId.createFromB58String('16Uiu2HAmVfV4GKQhdECMqYmUMGLy84RjTJQxTWDcmUX5847roBar'))

    const { native, hopr } = getAddresses(node)
    assert.equal(native, '0xEA9eDAE5CfC794B75C45c8fa89b605508A03742a')
    assert.equal(hopr, '16Uiu2HAmVfV4GKQhdECMqYmUMGLy84RjTJQxTWDcmUX5847roBar')
  })
})
