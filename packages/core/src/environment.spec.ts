import { resolveEnvironment } from './environment.js'
import assert from 'assert'

describe('test environment and flags', async function () {
  it('should resolve environment with a custom provider', async function () {
    const customProvider = 'https://a-dummy-provider.com'
    const environment_id = 'anvil-localhost'

    const resolvedEnvironment = resolveEnvironment(environment_id, customProvider)
    assert.equal(resolvedEnvironment.network.default_provider, customProvider, 'provider')
  })
})
