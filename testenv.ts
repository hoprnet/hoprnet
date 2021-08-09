import { validate } from 'jsonschema'
import fs from 'fs'

function validate_file(file_path: string, schema_path: string) {
    console.log(`testing ${file_path} against ${schema_path}`)
    const schema_content = fs.readFileSync(schema_path, 'utf-8')
    const schema = JSON.parse(schema_content)
    const data_content = fs.readFileSync(file_path, 'utf-8')
    const data = JSON.parse(data_content)

    const res = validate(data, schema)
    for (const err of res.errors) {
        console.log(err.stack)
    }
    if (res.errors.length > 0) {
        throw new Error(`validation failed`)
    }
}

validate_file('./environments.json', './environments_schema.json')
