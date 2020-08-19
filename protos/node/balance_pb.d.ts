// package: balance
// file: balance.proto

import * as jspb from "google-protobuf";

export class GetNativeBalanceRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetNativeBalanceRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetNativeBalanceRequest): GetNativeBalanceRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetNativeBalanceRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetNativeBalanceRequest;
  static deserializeBinaryFromReader(message: GetNativeBalanceRequest, reader: jspb.BinaryReader): GetNativeBalanceRequest;
}

export namespace GetNativeBalanceRequest {
  export type AsObject = {
  }
}

export class GetNativeBalanceResponse extends jspb.Message {
  getAmount(): string;
  setAmount(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetNativeBalanceResponse.AsObject;
  static toObject(includeInstance: boolean, msg: GetNativeBalanceResponse): GetNativeBalanceResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetNativeBalanceResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetNativeBalanceResponse;
  static deserializeBinaryFromReader(message: GetNativeBalanceResponse, reader: jspb.BinaryReader): GetNativeBalanceResponse;
}

export namespace GetNativeBalanceResponse {
  export type AsObject = {
    amount: string,
  }
}

export class GetHoprBalanceRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetHoprBalanceRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetHoprBalanceRequest): GetHoprBalanceRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetHoprBalanceRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetHoprBalanceRequest;
  static deserializeBinaryFromReader(message: GetHoprBalanceRequest, reader: jspb.BinaryReader): GetHoprBalanceRequest;
}

export namespace GetHoprBalanceRequest {
  export type AsObject = {
  }
}

export class GetHoprBalanceResponse extends jspb.Message {
  getAmount(): string;
  setAmount(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetHoprBalanceResponse.AsObject;
  static toObject(includeInstance: boolean, msg: GetHoprBalanceResponse): GetHoprBalanceResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetHoprBalanceResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetHoprBalanceResponse;
  static deserializeBinaryFromReader(message: GetHoprBalanceResponse, reader: jspb.BinaryReader): GetHoprBalanceResponse;
}

export namespace GetHoprBalanceResponse {
  export type AsObject = {
    amount: string,
  }
}

