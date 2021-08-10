import { validate } from 'jsonschema'
import fs from 'fs'

function load_json(file_path: string): any {
    const content = fs.readFileSync(file_path, 'utf-8')
    return JSON.parse(content)
}

function validate_data(data, schema: string) {
    const res = validate(data, schema)
    for (const err of res.errors) {
        console.log(err.stack)
    }
    if (res.errors.length > 0) {
        throw new Error(`validation failed`)
    }
}

const env_data = load_json('./environments.json')
const env_schema = load_json('./environments_schema.json')

console.log(`testing environments.json against schema`)
validate_data(env_data, env_schema)
console.log(`schema test ok`)

