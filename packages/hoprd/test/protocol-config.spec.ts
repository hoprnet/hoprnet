import { validate } from 'jsonschema'
import fs from 'fs'

type Network = {
  id: string
  description: string
  chain_id: number
  default_provider: string
  gas: string
  gas_multiplier: number
  native_token_name: string
  hopr_token_name: string
}

type Environment = {
  id: string
  network_id: string
  deploy_block: number
  token_contract_address: string
  channels_contract_address: string
}

type ProtocolConfig = {
  environments: Environment[]
  networks: Network[]
}

function load_json(file_path: string): ProtocolConfig {
  const content = fs.readFileSync(file_path, 'utf-8')
  return JSON.parse(content)
}

function validate_data(data: ProtocolConfig, schema: any) {
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
    const env_data = load_json('./protocol-config.json')
    const env_schema = load_json('./protocol-config-schema.json')

    validate_data(env_data, env_schema)
  })

  it('should be internally consistent', async function () {
    function get_network(id: string): Network | null {
      for (const network of env_data.networks) {
        if (network.id === id) {
          return network
        }
      }
      return null
    }

    const env_data = load_json('./protocol-config.json')

    for (const env of env_data.environments) {
      if (get_network(env.network_id) == null) {
        throw new Error(`no such network: ${env.network_id}`)
      }
    }
  })
})
