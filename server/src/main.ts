import { NestFactory } from '@nestjs/core'
import { Transport, MicroserviceOptions } from '@nestjs/microservices'
import { AppModule } from './app.module'
import { HOPR_PROTOS_FOLDER_DIR, PROTO_PACKAGES, PROTO_FILES } from './constants'

async function bootstrap() {
  console.log(':: HOPR Server Starting ::')

  const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
    transport: Transport.GRPC,
    options: {
      // @TODO: make this configurable
      url: '0.0.0.0:50051',
      package: PROTO_PACKAGES,
      protoPath: PROTO_FILES,
      loader: {
        includeDirs: [HOPR_PROTOS_FOLDER_DIR],
      },
    },
  })

  await app.listenAsync()
  console.log(':: HOPR Server Started ::')
}

bootstrap()

process.on('unhandledRejection', (error: Error) => {
  console.error(error)
  process.exit(1)
})

process.on('uncaughtException', (error: Error) => {
  console.error(error)
  process.exit(1)
})
