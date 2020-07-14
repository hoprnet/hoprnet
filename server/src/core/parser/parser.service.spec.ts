import PeerInfo from 'peer-info'
import { Test, TestingModule } from '@nestjs/testing'
import { ParserService } from './parser.service'

describe('ParserService', () => {
  let service: ParserService

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ParserService],
    }).compile()

    service = module.get<ParserService>(ParserService)
  })

  it('should parse bootstrap node', async () => {
    const peerId = '/ip4/34.65.114.152/tcp/9091/p2p/16Uiu2HAmQrtY26aYgLBUMjhw9qvZABKYTdncHR5VD4MDrMLVSpkp'
    const peerInfo = await service.parseBootstrap(peerId)
    expect(peerInfo).toBeInstanceOf(PeerInfo)
  })

  it('should fail on an invalid address and throw a rejected promise', async () => {
    await expect(async () => {
      const peerId = 'Invalid address'
      await service.parseBootstrap(peerId)
    }).rejects.toThrow()
  })
})
