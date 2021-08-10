import { validate } from 'jsonschema'
import fs from 'fs'

function load_json(file_path: string): any {
    const content = fs.readFileSync(file_path, 'utf-8')
    return JSON.parse(content)
}

function validate_file(file_path: string, schema_path: string) {
    console.log(`testing ${file_path} against ${schema_path}`)

    const data = load_json(file_path)
    const schema = load_json(schema_path)

    const res = validate(data, schema)
    for (const err of res.errors) {
        console.log(err.stack)
    }
    if (res.errors.length > 0) {
        throw new Error(`validation failed`)
    }
    console.log(`schema test ok`)
}

validate_file('./environments.json', './environments_schema.json')
