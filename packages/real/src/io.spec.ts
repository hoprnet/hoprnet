import fs from 'fs'
import { read_file } from './io'
import * as assert from 'assert'

describe('test io abstraction for real', async function () {
  it('test reading files', async function () {
    let file = 'package.json'
    let data = read_file(file)
    assert.deepEqual(data, fs.readFileSync(file))
  })
})
