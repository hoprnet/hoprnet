import { Test, TestingModule } from '@nestjs/testing'
import { ConfigService } from '@nestjs/config'
import { CoreService } from './core.service'
import { ParserService } from './parser/parser.service'
import { setNode } from '../main' 
import { resetNodeForTests } from './hoprnode'

describe('CoreService', () => {
  let service: CoreService

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ConfigService, ParserService, CoreService],
    }).compile()

    service = module.get<CoreService>(CoreService)
  })

  it('should be defined', () => {
    expect(service).toBeDefined()
  })

})

describe('We should be able to inject our own HOPR node into this nest stuff',  () =>{
  it('should work',async () => {
    let mockNode: any = jest.fn()
    mockNode.stop = jest.fn()

    setNode(mockNode)

    // Run all the nest boilerplate stuff.
    const testmod: TestingModule = await Test.createTestingModule({
      providers: [ConfigService, ParserService, CoreService],
    }).compile()
    let service = testmod.get<CoreService>(CoreService)

    service.start()
    service.stop()

    expect(mockNode.stop).toHaveBeenCalled()
    resetNodeForTests()
  })
})
