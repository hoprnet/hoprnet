import { Test, TestingModule } from '@nestjs/testing'
import { ConfigModule } from '@nestjs/config'
import { CoreModule } from '../core/core.module'
import { SystemModule } from '../system/system.module'
import { GrpcController } from './grpc.controller'
import { GrpcService } from './grpc.service'

describe('Grpc Controller', () => {
  let controller: GrpcController

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
      controllers: [GrpcController],
    }).compile()

    controller = module.get<GrpcController>(GrpcController)
  })

  it('should be defined', () => {
    expect(controller).toBeDefined()
  })
})
