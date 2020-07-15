import { Test } from '@nestjs/testing'
import { ConfigModule } from '@nestjs/config'
import { INestApplication } from '@nestjs/common'
import { Transport } from '@nestjs/microservices'
import * as grpc from 'grpc'
import { AppModule } from '../src/app.module'
import { HOPR_PROTOS_FOLDER_DIR, PROTO_PACKAGES, PROTO_FILES } from '../src/constants'
import { VersionRequest } from '@hoprnet/hopr-protos/node/version_pb'
import { VersionClient } from '@hoprnet/hopr-protos/node/version_grpc_pb'
import { StatusRequest } from '@hoprnet/hopr-protos/node/status_pb'
import { StatusClient } from '@hoprnet/hopr-protos/node/status_grpc_pb'
import { ShutdownRequest } from '@hoprnet/hopr-protos/node/shutdown_pb'
import { ShutdownClient } from '@hoprnet/hopr-protos/node/shutdown_grpc_pb'
import { PingClient } from '@hoprnet/hopr-protos/node/ping_grpc_pb'
import { PingRequest } from '@hoprnet/hopr-protos/node/ping_pb'
import { GetNativeAddressRequest, GetHoprAddressRequest } from '@hoprnet/hopr-protos/node/address_pb'
import { AddressClient } from '@hoprnet/hopr-protos/node/address_grpc_pb'

const BOOTSTRAP = {
  id: 0,
  nativeAddress: '0x92f84e4963dd31551927664007835b1908ac9020',
  hoprAddress: '16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX',
  serverHost: '0.0.0.0:50051',
  coreHost: '0.0.0.0:9091',
}

const NODE = {
  id: 1,
  nativeAddress: '0x32c160a5008e517ce06df4f7d4a39ffc52e049cf',
  hoprAddress: '16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65',
  serverHost: '0.0.0.0:50052',
  coreHost: '0.0.0.0:9092',
}

const SetupServer = async (serverOps: Record<string, any>, env: Record<string, any>): Promise<INestApplication> => {
  const TestModule = await Test.createTestingModule({
    imports: [
      ConfigModule.forRoot({
        isGlobal: true,
        load: [() => env],
      }),
      AppModule,
    ],
  }).compile()

  const app = TestModule.createNestApplication()
  app.connectMicroservice({
    transport: Transport.GRPC,
    options: serverOps,
  })

  await app.startAllMicroservicesAsync()
  await app.init()

  return app
}

const SetupClient = <T extends typeof grpc.Client>(Client: T, server: 'bootstrap' | 'node'): InstanceType<T> => {
  return (new Client(
    server === 'bootstrap' ? BOOTSTRAP.serverHost : NODE.serverHost,
    grpc.credentials.createInsecure(),
  ) as unknown) as InstanceType<T>
}

// @TODO: fix open handles
describe('GRPC transport', () => {
  let bootstrap: INestApplication
  let node: INestApplication

  beforeAll(async () => {
    bootstrap = await SetupServer(
      {
        url: BOOTSTRAP.serverHost,
        package: PROTO_PACKAGES,
        protoPath: PROTO_FILES,
        loader: {
          includeDirs: [HOPR_PROTOS_FOLDER_DIR],
        },
      },
      {
        DEBUG: true,
        ID: BOOTSTRAP.id,
        BOOTSTRAP_NODE: true,
        CORE_HOST: BOOTSTRAP.coreHost,
      },
    )

    node = await SetupServer(
      {
        url: NODE.serverHost,
        package: PROTO_PACKAGES,
        protoPath: PROTO_FILES,
        loader: {
          includeDirs: [HOPR_PROTOS_FOLDER_DIR],
        },
      },
      {
        DEBUG: true,
        ID: NODE.id,
        BOOTSTRAP_NODE: false,
        CORE_HOST: NODE.coreHost,
        BOOTSTRAP_SERVERS: ['/ip4/127.0.0.1/tcp/9093/p2p/16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX'],
      },
    )
  })

  afterAll(async () => {
    await node.close()
    await bootstrap.close()
  })

  it('should get status', async (done) => {
    const client = SetupClient(StatusClient, 'bootstrap')

    client.getStatus(new StatusRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.id).toBe(BOOTSTRAP.hoprAddress)
      expect(data.multiAddressesList.length).toBeGreaterThan(0)
      expect(data.connectedNodes).toBe(1)

      client.close()
      done()
    })
  })

  it('should get version', async (done) => {
    const client = SetupClient(VersionClient, 'bootstrap')

    client.getVersion(new VersionRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(typeof data.version).toBe('string')
      expect(data.componentsVersionMap).toHaveLength(5)

      client.close()
      done()
    })
  })

  it('node should ping bootstrap node', async (done) => {
    const client = SetupClient(PingClient, 'node')

    const req = new PingRequest()
    req.setPeerid(BOOTSTRAP.hoprAddress)

    client.ping(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(typeof data.latency).toBe('number')

      client.close()
      done()
    })
  })

  it('should get native address', async (done) => {
    const client = SetupClient(AddressClient, 'bootstrap')

    client.getNativeAddress(new GetNativeAddressRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.address).toBe(BOOTSTRAP.nativeAddress)

      client.close()
      done()
    })
  })

  it('should get HOPR address', async (done) => {
    const client = SetupClient(AddressClient, 'bootstrap')

    client.getHoprAddress(new GetHoprAddressRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.address).toBe(BOOTSTRAP.hoprAddress)

      client.close()
      done()
    })
  })

  // keep this last as it shutdowns the server
  it('should shutdown', async (done) => {
    const client = SetupClient(ShutdownClient, 'bootstrap')

    client.shutdown(new ShutdownRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.timestamp).toBeGreaterThan(0)

      client.close()
      done()
    })
  })
})
