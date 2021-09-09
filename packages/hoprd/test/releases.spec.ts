import { validate } from 'jsonschema'
import fs from 'fs'

describe('releases config', async function () {
  it('should conform to schema', async function () {
    const env_data = loadJson('./releases.json') as ProtocolConfig
    const env_schema = loadJson('./releases-schema.json')

    validateData(env_data, env_schema)
  })
})
