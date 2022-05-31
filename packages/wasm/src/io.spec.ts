import fs from 'fs'
import { read_file } from './io'
import * as assert from 'assert'

describe('test io abstraction for wasm', async function () {
  it('test reading files', async function () {
    let file = 'package.json'
    let data = read_file(file)

    assert.equal(false, data.hasError())
    assert.equal(undefined, data.error)
    assert.deepEqual(data.data , fs.readFileSync(file))
  })

  it('test reading error', async function () {
    let file = 'package-non-existent.json'
    let data = read_file(file)

    assert.equal(undefined, data.data)
    assert.equal(true, data.hasError())
    assert.notEqual(undefined, data.error)
  })
})
