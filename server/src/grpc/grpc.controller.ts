import { Controller } from '@nestjs/common'
import { GrpcMethod, RpcException } from '@nestjs/microservices'
import { status as STATUS } from 'grpc'
import { GrpcService } from './grpc.service'
import { StatusResponse } from '@hoprnet/hopr-protos/node/status_pb'
import { VersionResponse } from '@hoprnet/hopr-protos/node/version_pb'
import { ShutdownResponse } from '@hoprnet/hopr-protos/node/shutdown_pb'
import { PingRequest, PingResponse } from '@hoprnet/hopr-protos/node/ping_pb'
import { GetNativeAddressResponse, GetHoprAddressResponse } from '@hoprnet/hopr-protos/node/address_pb'
import { GetNativeBalanceResponse, GetHoprBalanceResponse } from '@hoprnet/hopr-protos/node/balance_pb'

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

  // @TODO: rename proto method from 'ping' to 'getPing'
  @GrpcMethod('Ping', 'ping')
  // @ts-ignore @TODO: protoc types do not match nestjs
  async getPing({ peerId }: PingRequest.AsObject): Promise<PingResponse.AsObject> {
    try {
      return this.grpcService.getPing(peerId)
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
      console.error(err)
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
      console.error(err)
      throw new RpcException({
        code: STATUS.INTERNAL,
        message: err,
      })
    }
  }
}
