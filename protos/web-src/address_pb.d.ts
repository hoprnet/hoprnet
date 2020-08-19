// package: address
// file: address.proto

import * as jspb from "google-protobuf";

export class GetNativeAddressRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetNativeAddressRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetNativeAddressRequest): GetNativeAddressRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetNativeAddressRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetNativeAddressRequest;
  static deserializeBinaryFromReader(message: GetNativeAddressRequest, reader: jspb.BinaryReader): GetNativeAddressRequest;
}

export namespace GetNativeAddressRequest {
  export type AsObject = {
  }
}

export class GetNativeAddressResponse extends jspb.Message {
  getAddress(): string;
  setAddress(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetNativeAddressResponse.AsObject;
  static toObject(includeInstance: boolean, msg: GetNativeAddressResponse): GetNativeAddressResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetNativeAddressResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetNativeAddressResponse;
  static deserializeBinaryFromReader(message: GetNativeAddressResponse, reader: jspb.BinaryReader): GetNativeAddressResponse;
}

export namespace GetNativeAddressResponse {
  export type AsObject = {
    address: string,
  }
}

export class GetHoprAddressRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetHoprAddressRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetHoprAddressRequest): GetHoprAddressRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetHoprAddressRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetHoprAddressRequest;
  static deserializeBinaryFromReader(message: GetHoprAddressRequest, reader: jspb.BinaryReader): GetHoprAddressRequest;
}

export namespace GetHoprAddressRequest {
  export type AsObject = {
  }
}

export class GetHoprAddressResponse extends jspb.Message {
  getAddress(): string;
  setAddress(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetHoprAddressResponse.AsObject;
  static toObject(includeInstance: boolean, msg: GetHoprAddressResponse): GetHoprAddressResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetHoprAddressResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetHoprAddressResponse;
  static deserializeBinaryFromReader(message: GetHoprAddressResponse, reader: jspb.BinaryReader): GetHoprAddressResponse;
}

export namespace GetHoprAddressResponse {
  export type AsObject = {
    address: string,
  }
}

