import { Module, OnModuleInit, OnModuleDestroy } from '@nestjs/common'
import { ConfigService } from '@nestjs/config'
import { ParserService } from './parser/parser.service'
import { CoreService } from './core.service'

@Module({
  providers: [ParserService, CoreService],
  exports: [CoreService],
})
export class CoreModule implements OnModuleInit, OnModuleDestroy {
  constructor(private configService: ConfigService, private coreService: CoreService) {}

  async onModuleInit(): Promise<void> {
    await this.coreService.start({
      debug: this.configService.get('debug'),
      id: this.configService.get('id'),
      bootstrapNode: this.configService.get('bootstrapNode'),
      host: this.configService.get('host'),
      bootstrapServers: this.configService.get('bootstrapServers'),
    })
  }

  async onModuleDestroy(): Promise<void> {
    await this.coreService.stop()
  }
}
