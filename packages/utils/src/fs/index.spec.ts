import assert from 'assert'

import { existsSync, mkdirSync } from 'fs'
import { clearDirectory } from '.'

describe('test clean directory', function () {
  it('should delete the directory structure recursively', function () {
    const directory = `filesystem_test/test/sth else/`
    mkdirSync(`${__dirname}/${directory}`, { recursive: true })

    assert(existsSync(`${__dirname}/${directory}`))

    clearDirectory(`${__dirname}/filesystem_test`)

    assert(!existsSync(`${__dirname}/filesystem_test`))

    assert.throws(() => clearDirectory(`${__dirname}/non_existing`))
    assert.throws(() => clearDirectory(`${__dirname}/non existing`))
  })
})
