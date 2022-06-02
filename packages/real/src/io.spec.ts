import fs from 'fs'
import { read_file } from './io'
import * as assert from 'assert'

describe('test io abstraction for real', async function () {
  it('test reading files', async function () {
    let file = 'package.json'
    let data = read_file(file)

    assert.equal(data.hasError(), false)
    assert.equal(data.error, undefined)
    assert.deepEqual(data.data, fs.readFileSync(file))
  })

  it('test reading error', async function () {
    let file = 'package-non-existent.json'
    let data = read_file(file)

    assert.equal(data.data, undefined)
    assert.equal(data.hasError(), true)
    assert.notEqual(data.error, undefined)
  })
})
