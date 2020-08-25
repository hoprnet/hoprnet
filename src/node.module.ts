import { Module, DynamicModule } from '@nestjs/common'
import Hopr from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

export const PROVIDER_NAME = 'HoprNode'

@Module({})
export class NodeModule {
  static register(options: { node: Hopr<HoprCoreConnector>; isGlobal: boolean }): DynamicModule {
    return {
      module: NodeModule,
      providers: [
        {
          provide: PROVIDER_NAME,
          useFactory: () => options.node,
        },
      ],
      exports: [PROVIDER_NAME],
      global: options.isGlobal,
    }
  }
}
