import assert from 'assert'

import { existsSync } from 'fs'
import { clearDirectory, createDirectoryIfNotExists } from '.'

describe('test clean directory', function () {
  it('should delete the directory structure recursively', function () {
    const directory = `filesystem_test/test/sth else/`
    createDirectoryIfNotExists(`${__dirname}/${directory}`)

    assert(existsSync(`${__dirname}/${directory}`))

    clearDirectory(`${__dirname}/filesystem_test`)

    assert(!existsSync(`${__dirname}/filesystem_test`))

    assert.throws(() => clearDirectory(`${__dirname}/non_existing`))
    assert.throws(() => clearDirectory(`${__dirname}/non existing`))
  })
})
