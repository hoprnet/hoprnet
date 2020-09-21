import { TextEncoder } from 'util'
import { Test } from '@nestjs/testing'
import { ConfigModule } from '@nestjs/config'
import { INestApplication } from '@nestjs/common'
import { Transport } from '@nestjs/microservices'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund } from '@hoprnet/hopr-ethereum'
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
import { ChannelsClient } from '@hoprnet/hopr-protos/node/channels_grpc_pb'
import {
  OpenChannelRequest,
  GetChannelsRequest,
  GetChannelDataRequest,
  CloseChannelRequest,
} from '@hoprnet/hopr-protos/node/channels_pb'
import { SendClient } from '@hoprnet/hopr-protos/node/send_grpc_pb'
import { SendRequest } from '@hoprnet/hopr-protos/node/send_pb'
import { ListenClient } from '@hoprnet/hopr-protos/node/listen_grpc_pb'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import { WithdrawClient } from '@hoprnet/hopr-protos/node/withdraw_grpc_pb'
import { WithdrawNativeRequest, WithdrawHoprRequest } from '@hoprnet/hopr-protos/node/withdraw_pb'
import { BalanceClient } from '@hoprnet/hopr-protos/node/balance_grpc_pb'
import { GetNativeBalanceRequest, GetHoprBalanceRequest } from '@hoprnet/hopr-protos/node/balance_pb'

// configuration for each node we are going to boot
const NODES: {
  [key in 'bootstrap' | 'alice' | 'bob']: {
    id: number
    nativeAddress: string
    hoprAddress: string
    serverHost: string
    coreHost: string
  }
} = {
  bootstrap: {
    id: 0,
    nativeAddress: '0x92f84e4963dd31551927664007835b1908ac9020',
    hoprAddress: '16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX',
    serverHost: '127.0.0.1:50051',
    coreHost: '127.0.0.1:9091',
  },
  alice: {
    id: 1,
    nativeAddress: '0x32c160a5008e517ce06df4f7d4a39ffc52e049cf',
    hoprAddress: '16Uiu2HAkzuoWfxBgsgBCr8xqpkjs1RAmtDPxafCUAcbBEonnVQ65',
    serverHost: '127.0.0.1:50052',
    coreHost: '127.0.0.1:9092',
  },
  bob: {
    id: 2,
    nativeAddress: '0xbd9c6A0b75F383FA94dCB8543CdEf83cFe74274B',
    hoprAddress: '16Uiu2HAmRfR1Qus69Lhn4t9gCkHjdsrWHpnwJwN7Dbjw197rZLqk',
    serverHost: '127.0.0.1:50053',
    coreHost: '127.0.0.1:9093',
  },
}

const SetupServer = async (serverOps: Record<string, any>, env: Record<string, any>): Promise<INestApplication> => {
  const TestModule = await Test.createTestingModule({
    imports: [
      ConfigModule.forRoot({
        isGlobal: true,
        load: [() => env],
      }),
      AppModule.register(),
    ],
  }).compile()

  const app = TestModule.createNestApplication()
  app.connectMicroservice({
    transport: Transport.GRPC,
    options: {
      package: PROTO_PACKAGES,
      protoPath: PROTO_FILES,
      loader: {
        includeDirs: [HOPR_PROTOS_FOLDER_DIR],
      },
      ...serverOps,
    },
  })

  await app.startAllMicroservicesAsync()
  await app.init()

  return app
}

const SetupClient = <T extends typeof grpc.Client>(Client: T, server: keyof typeof NODES): InstanceType<T> => {
  return (new Client(NODES[server].serverHost, grpc.credentials.createInsecure()) as unknown) as InstanceType<T>
}

// @TODO: fix open handles
describe('GRPC transport', () => {
  const aliceAndBobChannelId = '0x9a94c47dc86a1f724faaad7fd01532307f13855b2659144ea47d137e281f6f57'
  const aliceToBobMessage = new TextEncoder().encode('Hello Bob! this is Alice.')
  const bobToAliceMessage = new TextEncoder().encode('Hey Alice, do you like HOPR?')

  const ganache = new Ganache()
  let bootstrap: INestApplication
  let alice: INestApplication
  let bob: INestApplication

  beforeAll(async () => {
    jest.setTimeout(90e3)

    // setup blockchain, migrate contracts, fund accounts
    await ganache.start()
    await migrate()
    await fund()

    bootstrap = await SetupServer(
      {
        url: NODES.bootstrap.serverHost,
      },
      {
        DEBUG_MODE: true,
        ID: NODES.bootstrap.id,
        BOOTSTRAP_NODE: true,
        CORE_HOST: NODES.bootstrap.coreHost,
        PROVIDER: 'ws://127.0.0.1:9545/',
      },
    )

    alice = await SetupServer(
      {
        url: NODES.alice.serverHost,
      },
      {
        DEBUG_MODE: true,
        ID: NODES.alice.id,
        BOOTSTRAP_NODE: false,
        CORE_HOST: NODES.alice.coreHost,
        PROVIDER: 'ws://127.0.0.1:9545/',
        // here we are using port 9093 because we use demo accounts
        // see https://github.com/hoprnet/hopr-core/blob/0f455e27c8d117880eb0fcc91bf31ae3584f96c9/src/utils/libp2p/getPeerInfo.ts#L28
        BOOTSTRAP_SERVERS: ['/ip4/127.0.0.1/tcp/9093/p2p/16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX'],
      },
    )

    bob = await SetupServer(
      {
        url: NODES.bob.serverHost,
      },
      {
        DEBUG_MODE: true,
        ID: NODES.bob.id,
        BOOTSTRAP_NODE: false,
        CORE_HOST: NODES.bob.coreHost,
        PROVIDER: 'ws://127.0.0.1:9545/',
        // here we are using port 9093 because we use demo accounts
        // see https://github.com/hoprnet/hopr-core/blob/0f455e27c8d117880eb0fcc91bf31ae3584f96c9/src/utils/libp2p/getPeerInfo.ts#L28
        BOOTSTRAP_SERVERS: ['/ip4/127.0.0.1/tcp/9093/p2p/16Uiu2HAmNqLm83bwMq9KQEZEWHcbsHQfBkbpZx4eVSoDG4Mp6yfX'],
      },
    )
  })

  afterAll(async () => {
    await alice.close()
    await bob.close()
    await bootstrap.close()
    await ganache.stop()
  })

  it('should get status', async (done) => {
    const client = SetupClient(StatusClient, 'bootstrap')

    client.getStatus(new StatusRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.id).toBe(NODES.bootstrap.hoprAddress)
      expect(data.multiAddressesList.length).toBeGreaterThan(0)
      expect(data.connectedNodes).toBe(2)

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
    const client = SetupClient(PingClient, 'alice')

    const req = new PingRequest()
    req.setPeerId(NODES.bootstrap.hoprAddress)

    client.getPing(req, (err, res) => {
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
      expect(data.address).toBe(NODES.bootstrap.nativeAddress)

      client.close()
      done()
    })
  })

  it('should get HOPR address', async (done) => {
    const client = SetupClient(AddressClient, 'bootstrap')

    client.getHoprAddress(new GetHoprAddressRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.address).toBe(NODES.bootstrap.hoprAddress)

      client.close()
      done()
    })
  })

  it('should open channel', async (done) => {
    const client = SetupClient(ChannelsClient, 'alice')

    const req = new OpenChannelRequest()
    req.setPeerId(NODES.bob.hoprAddress)

    client.openChannel(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.channelId).toBe(aliceAndBobChannelId)

      client.close()
      done()
    })
  })

  it("should get Alice's and Bob's channel", async (done) => {
    const client = SetupClient(ChannelsClient, 'alice')

    client.getChannels(new GetChannelsRequest(), (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.channelsList).toHaveLength(1)
      expect(data.channelsList[0]).toBe(aliceAndBobChannelId)

      client.close()
      done()
    })
  })

  it('should get channel information', async (done) => {
    const client = SetupClient(ChannelsClient, 'alice')

    const req = new GetChannelDataRequest()
    req.setChannelId(aliceAndBobChannelId)

    client.getChannelData(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.balance).toBe('20')
      expect(data.state).toBe(0)

      client.close()
      done()
    })
  })

  it('alice should send a message to bob using no intermediate nodes', async (done) => {
    const client = SetupClient(SendClient, 'alice')

    const req = new SendRequest()
    req.setPeerId(NODES.bob.hoprAddress)
    req.setPayload(aliceToBobMessage)

    client.send(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.intermediatePeerIdsList).toHaveLength(0)

      client.close()
      done()
    })
  })

  it('alice should send a message to bob using 1 intermediate node', async (done) => {
    const client = SetupClient(SendClient, 'alice')

    const req = new SendRequest()
    req.setPeerId(NODES.bob.hoprAddress)
    req.setPayload(aliceToBobMessage)
    req.setIntermediatePeerIdsList([NODES.bootstrap.hoprAddress])

    client.send(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.intermediatePeerIdsList).toHaveLength(1)

      client.close()
      done()
    })
  })

  it('alice should receive message from bob', async (done) => {
    const alice = SetupClient(ListenClient, 'alice')
    const bob = SetupClient(SendClient, 'bob')

    const listenReq = new ListenRequest()
    listenReq.setPeerId(NODES.bob.hoprAddress)

    const stream = alice.listen(listenReq)

    stream.on('data', (data) => {
      const [payload] = data.array

      const res = new ListenResponse()
      res.setPayload(payload)

      expect(res.getPayload_asU8()).toStrictEqual(bobToAliceMessage)

      alice.close()
      done()
    })

    const sendReq = new SendRequest()
    sendReq.setPeerId(NODES.alice.hoprAddress)
    sendReq.setPayload(bobToAliceMessage)

    bob.send(sendReq, (err) => {
      expect(err).toBeFalsy()

      bob.close()
    })
  })

  // @TODO: return transaction hash
  it("should close Alice's and Bob's channel", async (done) => {
    const client = SetupClient(ChannelsClient, 'alice')

    const req = new CloseChannelRequest()
    req.setChannelId(aliceAndBobChannelId)

    client.closeChannel(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(data.channelId).toBe(aliceAndBobChannelId)

      client.close()
      done()
    })
  })

  it("should get Alice's NATIVE balance", async (done) => {
    const client = SetupClient(BalanceClient, 'alice')

    const req = new GetNativeBalanceRequest()

    client.getNativeBalance(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(Number(data.amount)).toBeGreaterThan(0)

      client.close()
      done()
    })
  })

  it("should get Alice's HOPR balance", async (done) => {
    const client = SetupClient(BalanceClient, 'alice')

    const req = new GetHoprBalanceRequest()

    client.getHoprBalance(req, (err, res) => {
      expect(err).toBeFalsy()

      const data = res.toObject()
      expect(Number(data.amount)).toBeGreaterThan(0)

      client.close()
      done()
    })
  })

  it('should withdraw 1 wei from Alice', async (done) => {
    const client = SetupClient(WithdrawClient, 'alice')

    const req = new WithdrawNativeRequest()
    req.setRecipient(NODES.alice.nativeAddress)
    req.setAmount('1')

    client.withdrawNative(req, (err, res) => {
      expect(err).toBeFalsy()

      client.close()
      done()
    })
  })

  it('should withdraw 1 HOPR wei from Alice', async (done) => {
    const client = SetupClient(WithdrawClient, 'alice')

    const req = new WithdrawHoprRequest()
    req.setRecipient(NODES.alice.nativeAddress)
    req.setAmount('1')

    client.withdrawHopr(req, (err, res) => {
      expect(err).toBeFalsy()

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
