import { NestFactory } from '@nestjs/core'
import { ConfigService } from '@nestjs/config'
import { LoggerService } from '@nestjs/common'
import { Transport, MicroserviceOptions } from '@nestjs/microservices'
import { AppModule } from './app.module'
import { HOPR_PROTOS_FOLDER_DIR, PROTO_PACKAGES, PROTO_FILES } from './constants'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import Hopr from '@hoprnet/hopr-core'
import dotenvParseVariables from 'dotenv-parse-variables'

export type ServerOpts = {
  host?: string
  logger?: LoggerService
}

export async function startServer(node?: Hopr<HoprCoreConnector>, opts?: ServerOpts) {
  console.log(':: HOPR Server Starting ::')
  const host = opts && opts.host ? opts.host : '0.0.0.0:50051'
  const app = await NestFactory.createMicroservice<MicroserviceOptions>(
    AppModule.register({
      node,
    }),
    {
      transport: Transport.GRPC,
      logger: opts && opts.logger ? opts.logger : undefined,
      options: {
        url: host,
        package: PROTO_PACKAGES,
        protoPath: PROTO_FILES,
        loader: {
          includeDirs: [HOPR_PROTOS_FOLDER_DIR],
        },
      },
    },
  )

  app.enableShutdownHooks()
  await app.listenAsync()
  console.log(`:: HOPR Server Started at ${host} ::`)
}

// If module is run as main (ie. from command line)
if (typeof module !== 'undefined' && !module.parent) {
  process.on('unhandledRejection', (error: Error) => {
    console.error(error)
    // process.exit(1)
  })

  process.on('uncaughtException', (error: Error) => {
    console.error(error)
    // process.exit(1)
  })

  const envOptions = dotenvParseVariables({
    host: process.env.SERVER_HOST,
  }) as ServerOpts

  startServer(undefined, envOptions)
}
