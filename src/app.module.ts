import { Module } from '@nestjs/common'
import { ConfigModule } from '@nestjs/config'
import { GrpcModule } from './grpc/grpc.module'

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    GrpcModule,
  ],
})
export class AppModule {}
