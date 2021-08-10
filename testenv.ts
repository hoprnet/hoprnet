import { validate } from 'jsonschema'
import fs from 'fs'

const env_data = load_json('./environments.json')
const env_schema = load_json('./environments_schema.json')

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

type Environments = {
    environments: Environment[],
    networks: Network[]
}

function load_json(file_path: string): Environments {
    const content = fs.readFileSync(file_path, 'utf-8')
    return JSON.parse(content)
}

function validate_data(data: Environments, schema: any) {
    const res = validate(data, schema)
    for (const err of res.errors) {
        console.log(err.stack)
    }
    if (res.errors.length > 0) {
        throw new Error(`validation failed`)
    }
}

function get_network(id: string): Network | null {
    for (const network of env_data.networks) {
        if (network.id === id) {
            return network
        }
    }
    return null
}

console.log(`testing environments.json against schema`)
validate_data(env_data, env_schema)
console.log(`schema test ok`)

console.log(`testing environments.json integrity`)
for (const env of env_data.environments) {
    if (get_network(env.network_id) == null) {
        throw new Error(`no such network: ${env.network_id}`)
    }
}

console.log(`integrity test ok`)

