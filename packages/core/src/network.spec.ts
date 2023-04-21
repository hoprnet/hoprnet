import { resolveNetwork, supportedNetworks } from './network.js'
import assert from 'assert'

describe('test network and flags', async function () {
  it('should resolve network with a custom provider', async function () {
    const customProvider = 'https://a-dummy-provider.com'
    const network_name = 'anvil-localhost'

    const resolvedNetwork = resolveNetwork(network_name, customProvider)
    assert.equal(resolvedNetwork.chain.default_provider, customProvider, 'provider')
  })

  it('should get supported networks', function () {
    // Assuming that `anvil-localhost` is always supported
    assert(supportedNetworks().length > 0)
  })
})
