import * as mod from './index'
import assert from 'assert'
// @ts-ignore
import sinon from 'sinon'

const assertMatch = async (cmds, command, pattern: RegExp) => {
  let response = ''
  await cmds.execute((l) => (response += l), command)
  assert(response.match(pattern), `executing: (${command}) => ${response} should match ${pattern}`)
}

let mockNode = sinon.fake() as any

describe('Commands', () => {
  it('can import commands', () => {
    assert(mod)
  })

  it('can construct Commands object', () => {
    assert(mod.Commands)
    let cmds = new mod.Commands(mockNode)
    assert(cmds)
  })

  it('responds to nonsense commands', async () => {
    assert(mod.Commands)
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'not-a-real-command', /unknown command/i)
  })

  it('ping', async () => {
    mockNode.ping = sinon.fake.returns({ info: '', latency: 10 })
    let mockPeerId = '16Uiu2HAkyXRaL7fKu4qcjaKuo4WXizrpK63Ltd6kG2tH6oSV58AW'
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, `ping  ${mockPeerId}`, /pong/i)
    assert(mockNode.ping.calledOnce)
  })

  it('help', async () => {
    let mockNode: any = sinon.fake()
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'help', /help/)
  })

  it('send message', async () => {
    mockNode.sendMessage = sinon.fake()
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'send 16Uiu2HAmAJStiomwq27Kkvtat8KiEHLBSnAkkKCqZmLYKVLtkiB7 Hello, world', /.*/)
    assert(mockNode.sendMessage.calledOnce, 'send message not called')
    await assertMatch(cmds, 'send unknown-alias Hello, world', /invalid/i)

    await assertMatch(cmds, 'alias 16Uiu2HAmAJStiomwq27Kkvtat8KiEHLBSnAkkKCqZmLYKVLtkiB7 test', /.*/)
    await assertMatch(cmds, 'alias 16Uiu2HAkyXRaL7fKu4qcjaKuo4WXizrpK63Ltd6kG2tH6oSV58AW test2', /.*/)
    await assertMatch(cmds, 'send test,test2 Hello, world', /.*/)
    assert(mockNode.sendMessage.calledTwice, 'send message not called')
    await assertMatch(cmds, 'send ,test2 Hello, world', /.*/)
    assert(mockNode.sendMessage.callCount == 3, 'send message not called x3')
  })

  it('settings', async () => {
    let mockNode: any = sinon.fake()
    mockNode.getChannelStrategy = (): string => ''
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'settings', /includeRecipient/)
  })

  it('settings includeRecipient', async () => {
    let mockNode: any = sinon.fake()
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'settings includeRecipient', /false/)
  })

  it('alias addresses', async () => {
    let mockNode: any = sinon.fake()
    mockNode.sendMessage = sinon.fake()
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'alias', /No aliases found/)
    await assertMatch(cmds, 'alias 16Uiu2HAmQDFS8a4Bj5PGaTqQLME5SZTRNikz9nUPT3G4T6YL9o7V test', /.*/)
    await assertMatch(cmds, 'alias', /test/)
    await assertMatch(cmds, 'send test Hello, world', /.*/)
    assert(mockNode.sendMessage.calledOnce)
  })

  it('fund channel', async () => {
    const channelId = '16Uiu2HAmAJStiomwq27Kkvtat8KiEHLBSnAkkKCqZmLYKVLtkiB7'
    let mockNode: any = sinon.fake()
    mockNode.fundChannel = sinon.fake(async () => ({
      channelId: {
        toHex: () => channelId
      }
    }))

    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, `fund ${channelId} 10 15`, /Successfully funded channel/)
    await assertMatch(cmds, `fund ${channelId} 10`, /usage:/)
    await assertMatch(cmds, `fund ${channelId} 10 y`, /is not a number/)
  })

  it('close channel', async () => {
    let mockNode: any = sinon.fake()
    mockNode.smartContractInfo = async () => ({
      channelClosureSecs: 300
    })
    mockNode.closeChannel = sinon.fake(async () => ({
      status: undefined
    }))
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'close 16Uiu2HAmAJStiomwq27Kkvtat8KiEHLBSnAkkKCqZmLYKVLtkiB7', /Initiated channel closure/)
    await assertMatch(cmds, 'close 16Uiu2HAmAJStiomwq27Kkvtat8KiEHLBSnAkkKCqZmLYKVLtkiB7', /5 minutes/)
  })

  it('cover traffic', async () => {
    var clock = sinon.useFakeTimers(Date.now())
    let mockNode: any = sinon.fake()
    let receive
    mockNode.on = (ev, f) => {
      assert(ev == 'hopr:message')
      receive = f
    }
    mockNode.getId = () => 'node'
    mockNode.sendMessage = (m, id) => {
      assert(m, 'send message takes message')
      assert(id == mockNode.getId(), 'sends to self')
      setTimeout(() => receive(m), 300)
      return Promise.resolve()
    }
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'covertraffic start', /started/)
    await clock.tickAsync(30_000)
    await assertMatch(cmds, 'covertraffic stop', /stopped/)
    await assertMatch(cmds, 'covertraffic stats', /messages sent/)
    await assertMatch(cmds, 'covertraffic stats', /reliability/)

    clock.restore()
  })

  it('info', async () => {
    let mockNode: any = sinon.fake()
    mockNode.getAnnouncedAddresses = async () => []
    mockNode.getListeningAddresses = () => []
    mockNode.smartContractInfo = async () => ({
      channelClosureSecs: 300
    })
    let cmds = new mod.Commands(mockNode)
    await assertMatch(cmds, 'info', /Channel closure period: 5 minutes/)
  })
})
