import fs from 'fs'
import { read_file } from './io'
import * as assert from 'assert'

describe('test io abstraction for wasm', async function () {
  it('test reading files', async function () {
      let file = "../package.json"
      assert.strictEqual(read_file(file), fs.readFileSync(file))
  })
})