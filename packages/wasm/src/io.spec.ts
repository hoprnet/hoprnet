import fs from 'fs'
import { read_file } from './io'
import * as assert from 'assert'

describe('test io abstraction for wasm', async function () {
  it('test reading files', async function () {
      let file = "package.json"
      assert.deepEqual(read_file(file), new Uint8Array(fs.readFileSync(file)))
  })
})