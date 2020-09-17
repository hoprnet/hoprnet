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
  GetChannelDataRequest,
  GetChannelDataResponse,
  OpenChannelRequest,
  OpenChannelResponse,
  CloseChannelResponse,
  CloseChannelRequest,
} from '@hoprnet/hopr-protos/node/channels_pb'
import { SendRequest, SendResponse } from '@hoprnet/hopr-protos/node/send_pb'
import { ListenRequest, ListenResponse } from '@hoprnet/hopr-protos/node/listen_pb'
import {
  WithdrawNativeRequest,
  WithdrawHoprRequest,
  WithdrawNativeResponse,
  WithdrawHoprResponse,
} from '@hoprnet/hopr-protos/node/withdraw_pb'

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

  @GrpcMethod('Channels')
  async getChannelData(req: GetChannelDataRequest.AsObject): Promise<GetChannelDataResponse.AsObject> {
    try {
      return this.grpcService.getChannelData(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

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

  @GrpcMethod('Withdraw')
  async withdrawNative(req: WithdrawNativeRequest.AsObject): Promise<WithdrawNativeResponse.AsObject> {
    try {
      return this.grpcService.withdrawNative(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }

  @GrpcMethod('Withdraw')
  async withdrawHopr(req: WithdrawHoprRequest.AsObject): Promise<WithdrawHoprResponse.AsObject> {
    try {
      return this.grpcService.withdrawHopr(req)
    } catch (err) {
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }
}
