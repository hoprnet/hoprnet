import * as mod from './index'
import assert from 'assert'
import sinon from 'sinon'

const assertMatch = async (cmds: mod.CommandsV2, command: string, pattern: RegExp) => {
  let response = ''
  await cmds.execute((log: string) => (response += log), command)
  assert(response.match(pattern), `executing: (${command}) => ${response} should match ${pattern}`)
}

let mockNode = sinon.fake() as any

describe('Commands', () => {
  it('can import commands', () => {
    assert(mod)
  })

  it('can construct Commands object', () => {
    assert(mod.CommandsV2)
    let cmds = new mod.CommandsV2(mockNode)
    assert(cmds)
  })

  it('responds to nonsense commands', async () => {
    assert(mod.CommandsV2)
    let cmds = new mod.CommandsV2(mockNode)
    await assertMatch(cmds, 'not-a-real-command', /unknown command/i)
  })

  it('alias addresses', async () => {
    let mockNode: any = sinon.fake()
    mockNode.sendMessage = sinon.fake()
    let cmds = new mod.CommandsV2(mockNode)
    await assertMatch(cmds, 'alias', /No aliases found/)
    await assertMatch(cmds, 'alias 16Uiu2HAmQDFS8a4Bj5PGaTqQLME5SZTRNikz9nUPT3G4T6YL9o7V test', /.*/)
    await assertMatch(cmds, 'alias', /test/)
    await assertMatch(cmds, 'send test Hello, world', /.*/)
    // for some reason it should call sendMessage once and for some reason its not calling it
    assert(mockNode.sendMessage.calledOnce)
  })
})
