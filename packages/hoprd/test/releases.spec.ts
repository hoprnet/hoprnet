import { loadJson, validateData } from '@hoprnet/hopr-utils'

describe('releases config', async function () {
  it('should conform to schema', async function () {
    const env_data = loadJson('./releases.json')
    const env_schema = loadJson('./releases-schema.json')

    validateData(env_data, env_schema)
  })
})
