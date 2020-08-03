import { Module, OnModuleInit, OnModuleDestroy } from '@nestjs/common'
import { ConfigService } from '@nestjs/config'
import { ParserService } from './parser/parser.service'
import { CoreService } from './core.service'

@Module({
  providers: [ConfigService, ParserService, CoreService],
  exports: [CoreService],
})
export class CoreModule implements OnModuleInit, OnModuleDestroy {
  constructor(private coreService: CoreService) {}

  async onModuleInit(): Promise<void> {
    await this.coreService.start()
  }

  async onModuleDestroy(): Promise<void> {
    await this.coreService.stop()
  }
}
