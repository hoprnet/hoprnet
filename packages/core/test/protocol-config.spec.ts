import type { ProtocolConfig } from '../src/environment'
import { loadJson, validateData } from '@hoprnet/hopr-utils'
import semver from 'semver'

describe('protocol config', async function () {
  const data = loadJson('./protocol-config.json') as ProtocolConfig
  const schema = loadJson('./protocol-config-schema.json')

  it('should conform to schema', async function () {
    validateData(data, schema)
  })

  it('should be internally consistent', async function () {
    for (const env of Object.values(data.environments)) {
      if (!data.networks[env.chain]) {
        throw new Error(`no such network: ${env.chain}`)
      }
    }
  })

  it('should contain valid version ranges', async function () {
    for (const [id, env] of Object.entries(data.environments)) {
      if (!semver.validRange(env.version_range)) {
        throw new Error(`invalid range in env '${id}': '${env.version_range}'`)
      }
    }
  })
})
