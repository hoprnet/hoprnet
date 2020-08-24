import { Module, DynamicModule } from '@nestjs/common'
import { CoreModule } from '../core/core.module'
import { SystemModule } from '../system/system.module'
import { GrpcService } from './grpc.service'
import { GrpcController } from './grpc.controller'
import Hopr from '@hoprnet/hopr-core'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

@Module({})
export class GrpcModule {
  static register(node: Hopr<HoprCoreConnector>): DynamicModule {
    return {
      module: GrpcModule,
      imports: [CoreModule.register(node), SystemModule],
      providers: [GrpcService],
      controllers: [GrpcController],
    }
  }
}
