import { validate } from 'jsonschema'
import fs from 'fs'
import type { Network, ProtocolConfig } from '@hoprnet/hopr-core'

function loadJson(file_path: string): any {
  const content = fs.readFileSync(file_path, 'utf-8')
  return JSON.parse(content)
}

function validateData(data: ProtocolConfig, schema: any) {
  const res = validate(data, schema)
  for (const err of res.errors) {
    console.log(err.stack)
  }
  if (res.errors.length > 0) {
    throw new Error(`validation failed`)
  }
}

describe('protocol config', async function () {
  it('should conform to schema', async function () {
    const env_data = loadJson('./protocol-config.json') as ProtocolConfig
    const env_schema = loadJson('./protocol-config-schema.json')

    validateData(env_data, env_schema)
  })

  it('should be internally consistent', async function () {
    function getNetwork(id: string): Network | null {
      for (const network of env_data.networks) {
        if (network.id === id) {
          return network
        }
      }
      return null
    }

    const env_data = loadJson('./protocol-config.json') as ProtocolConfig

    for (const env of env_data.environments) {
      if (getNetwork(env.network_id) == null) {
        throw new Error(`no such network: ${env.network_id}`)
      }
    }
  })
})
