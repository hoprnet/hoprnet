import * as root from '../index'
const mod = root.commands

describe('Commands', () => {

  it('can import commands', () => {
    expect(mod).toBeDefined()
  }) 

  it('can construct Commands object', () => {
    let mockNode: any = jest.fn()
    expect(mod.Commands).toBeDefined()
    let cmds = new mod.Commands(mockNode)
    expect(cmds).toBeTruthy()
  })

  it('ping', async () => {
    let mockNode: any = jest.fn()
    mockNode.bootstrapServers = []
    mockNode.ping = jest.fn()
    let mockPeerId = '16Uiu2HAkyXRaL7fKu4qcjaKuo4WXizrpK63Ltd6kG2tH6oSV58AW'

    let cmds = new mod.Commands(mockNode)
    await cmds.execute(`ping  ${mockPeerId}`)
    expect(mockNode.ping).toHaveBeenCalled()
  })


  it('commands can save state', async () => {
    let mockNode: any = jest.fn()
    let cmds = new mod.Commands(mockNode)

    let ir = await cmds.execute('settings includeRecipient')
    expect(ir).toMatch(/false/)
    await cmds.execute('includeRecipient true')
    ir = await cmds.execute('settings includeRecipient')
    expect(ir).toMatch(/true/)
    await cmds.execute('includeRecipient false')
    ir = await cmds.execute('settings includeRecipient')
    expect(ir).toMatch(/false/)
  })

  it('send message', async () => {
    let mockNode: any = jest.fn()
    mockNode.sendMessage = jest.fn()
    let cmds = new mod.Commands(mockNode)
    await cmds.execute('send 16Uiu2HAmAJStiomwq27Kkvtat8KiEHLBSnAkkKCqZmLYKVLtkiB7 Hello, world')
    expect(mockNode.sendMessage).toHaveBeenCalled()
  })
})

