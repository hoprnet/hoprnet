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
    expect(cmds.crawl).toBeTruthy()
  })

  it('ping', async () => {
    let mockNode: any = jest.fn()
    mockNode.bootstrapServers = []
    mockNode.ping = jest.fn()
    let mockPeerId = '16Uiu2HAkyXRaL7fKu4qcjaKuo4WXizrpK63Ltd6kG2tH6oSV58AW'

    let cmds = new mod.Commands(mockNode)
    await cmds.ping.execute(mockPeerId)
    expect(mockNode.ping).toHaveBeenCalled()

  })

})

