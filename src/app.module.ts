import { Module, DynamicModule } from '@nestjs/common'
import { ConfigModule } from '@nestjs/config'
import { GrpcModule } from './grpc/grpc.module'
import { NodeModule } from './node.module'
import Hopr from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

@Module({})
export class AppModule {
  static register(
    options: {
      node?: Hopr<HoprCoreConnector>
    } = {},
  ): DynamicModule {
    return {
      module: AppModule,
      imports: [
        ConfigModule.forRoot({
          isGlobal: true,
        }),
        NodeModule.register({
          node: options.node,
          isGlobal: true,
        }),
        GrpcModule,
      ],
    }
  }
}
