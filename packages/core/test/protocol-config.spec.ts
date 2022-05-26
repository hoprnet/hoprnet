import type { ProtocolConfig } from '../src/environment'
import { loadJson, validateData } from '@hoprnet/hopr-utils'

describe('protocol config', async function () {
  it('should conform to schema', async function () {
    const env_data = loadJson('./protocol-config.json') as ProtocolConfig
    const env_schema = loadJson('./protocol-config-schema.json')

    validateData(env_data, env_schema)
  })

  it('should be internally consistent', async function () {
    const protocolConfig = loadJson('./protocol-config.json') as ProtocolConfig
    for (const env of Object.values(protocolConfig.environments)) {
      if (!protocolConfig.networks[env.network_id]) {
        throw new Error(`no such network: ${env.network_id}`)
      }
    }
  })
})
