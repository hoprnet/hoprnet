import assert from 'assert'
var mod: any

describe('test chat commands can be imported as a module', () => {
  it('can import chat without starting a node', () => {
    mod = require('./index') as any
  })

  it('can import commands', () => {
    assert(mod.commands)
  })
})
