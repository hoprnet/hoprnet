import { Module, OnModuleInit, OnModuleDestroy, DynamicModule } from '@nestjs/common'
import { ConfigService } from '@nestjs/config'
import { ParserService } from './parser/parser.service'
import { CoreService } from './core.service'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import Hopr from '@hoprnet/hopr-core'

@Module({})
export class CoreModule implements OnModuleInit, OnModuleDestroy {
  constructor(private coreService: CoreService) {}

  async onModuleInit(): Promise<void> {
    await this.coreService.start()
  }

  async onModuleDestroy(): Promise<void> {
    await this.coreService.stop()
  }
  static register(node: Hopr<HoprCoreConnector>): DynamicModule {
    return {
      module: CoreModule,
      providers: [ConfigService, ParserService, CoreService, {
        provide: 'HoprNode',
        useValue: node
      }],
      exports: [CoreService]
    }
  }
}
