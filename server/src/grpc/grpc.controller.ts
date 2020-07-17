import { Controller } from '@nestjs/common'
import { GrpcMethod, RpcException } from '@nestjs/microservices'
import { status as STATUS } from 'grpc'
import { Subject, Observable } from 'rxjs'
import { GrpcService } from './grpc.service'
import { StatusResponse } from '@hoprnet/hopr-protos/node/status_pb'
import { VersionResponse } from '@hoprnet/hopr-protos/node/version_pb'
import { ShutdownResponse } from '@hoprnet/hopr-protos/node/shutdown_pb'
import { PingRequest, PingResponse } from '@hoprnet/hopr-protos/node/ping_pb'
import { GetNativeAddressResponse, GetHoprAddressResponse } from '@hoprnet/hopr-protos/node/address_pb'
import { GetNativeBalanceResponse, GetHoprBalanceResponse } from '@hoprnet/hopr-protos/node/balance_pb'
import {
  GetChannelsResponse,
  GetChannelInfoRequest,
  GetChannelInfoResponse,
  OpenChannelRequest,
  OpenChannelResponse,
  CloseChannelResponse,
  CloseChannelRequest,
} from '@hoprnet/hopr-protos/node/channels_pb'
import { SendRequest, SendResponse } from '@hoprnet/hopr-protos/node/send_pb'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'

// @TODO: capture errors and turn them into GRPC errors
@Controller('grpc')
export class GrpcController {
  constructor(private grpcService: GrpcService) {}

  @GrpcMethod('Status')
  async getStatus(): Promise<StatusResponse.AsObject> {
    try {
      return this.grpcService.getStatus()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Version')
  async getVersion(): Promise<VersionResponse.AsObject> {
    try {
      return this.grpcService.getVersion()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Shutdown')
  async shutdown(): Promise<ShutdownResponse.AsObject> {
    try {
      return this.grpcService.shutdown()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Ping')
  async getPing(req: PingRequest.AsObject): Promise<PingResponse.AsObject> {
    try {
      return this.grpcService.getPing(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Balance')
  async getNativeBalance(): Promise<GetNativeBalanceResponse.AsObject> {
    try {
      return this.grpcService.getNativeBalance()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Balance')
  async getHoprBalance(): Promise<GetHoprBalanceResponse.AsObject> {
    try {
      return this.grpcService.getHoprBalance()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Address')
  async getNativeAddress(): Promise<GetNativeAddressResponse.AsObject> {
    try {
      return this.grpcService.getNativeAddress()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Address')
  async getHoprAddress(): Promise<GetHoprAddressResponse.AsObject> {
    try {
      return this.grpcService.getHoprAddress()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Channels')
  async getChannels(): Promise<GetChannelsResponse.AsObject> {
    try {
      return this.grpcService.getChannels()
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  // @TODO: rename 'getChannelInfo' to 'getChannel'
  // @TODO: rename 'req.channelid' to 'channelId'
  @GrpcMethod('Channels')
  async getChannelInfo(req: GetChannelInfoRequest.AsObject): Promise<GetChannelInfoResponse.AsObject> {
    try {
      return this.grpcService.getChannel(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  // @TODO: rename 'req.peerid' to 'peerId'
  @GrpcMethod('Channels')
  async openChannel(req: OpenChannelRequest.AsObject): Promise<OpenChannelResponse.AsObject> {
    try {
      return this.grpcService.openChannel(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  // @TODO: rename 'req.channelid' to 'channelId'
  @GrpcMethod('Channels')
  async closeChannel(req: CloseChannelRequest.AsObject): Promise<CloseChannelResponse.AsObject> {
    try {
      return this.grpcService.closeChannel(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Send')
  async send(req: SendRequest.AsObject): Promise<SendResponse.AsObject> {
    try {
      return this.grpcService.send(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  // here we need to use 'GrpcMethod' see: https://github.com/nestjs/nest/issues/2659#issuecomment-516164027
  @GrpcMethod('Listen')
  async listen(req: ListenRequest.AsObject): Promise<Observable<ListenResponse.AsObject>> {
    const events = await this.grpcService.listen(req)
    const subject = new Subject<ListenResponse.AsObject>()

    events.on('message', (message) => {
      subject.next({
        payload: message,
      })
    })

    return subject.asObservable()
  }
}
