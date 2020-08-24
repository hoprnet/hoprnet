import { Module, DynamicModule } from '@nestjs/common'
import { ConfigModule } from '@nestjs/config'
import { GrpcModule } from './grpc/grpc.module'
import Hopr from '@hoprnet/hopr-core'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

@Module({})
export class AppModule {
  static register(node: Hopr<HoprCoreConnector>): DynamicModule {
    return {
      module: AppModule,
      imports: [
        ConfigModule.forRoot({
          isGlobal: true,
        }),
        GrpcModule.register(node),
      ],
    }
  }
}
