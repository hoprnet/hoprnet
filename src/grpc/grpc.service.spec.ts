import { Test, TestingModule } from '@nestjs/testing'
import { ConfigModule } from '@nestjs/config'
import { CoreModule } from '../core/core.module'
import { SystemModule } from '../system/system.module'
import { GrpcService } from './grpc.service'

describe('GrpcService', () => {
  let service: GrpcService

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      imports: [
        ConfigModule.forRoot({
          isGlobal: true,
        }),
        CoreModule,
        SystemModule,
      ],
      providers: [GrpcService],
    }).compile()

    service = module.get<GrpcService>(GrpcService)
  })

  it('should be defined', () => {
    expect(service).toBeDefined()
  })
})
