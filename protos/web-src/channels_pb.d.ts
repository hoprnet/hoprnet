// package: channels
// file: channels.proto

import * as jspb from "google-protobuf";

export class GetChannelsRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetChannelsRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetChannelsRequest): GetChannelsRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetChannelsRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetChannelsRequest;
  static deserializeBinaryFromReader(message: GetChannelsRequest, reader: jspb.BinaryReader): GetChannelsRequest;
}

export namespace GetChannelsRequest {
  export type AsObject = {
  }
}

export class GetChannelsResponse extends jspb.Message {
  clearChannelsList(): void;
  getChannelsList(): Array<string>;
  setChannelsList(value: Array<string>): void;
  addChannels(value: string, index?: number): string;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetChannelsResponse.AsObject;
  static toObject(includeInstance: boolean, msg: GetChannelsResponse): GetChannelsResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetChannelsResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetChannelsResponse;
  static deserializeBinaryFromReader(message: GetChannelsResponse, reader: jspb.BinaryReader): GetChannelsResponse;
}

export namespace GetChannelsResponse {
  export type AsObject = {
    channelsList: Array<string>,
  }
}

export class GetChannelDataRequest extends jspb.Message {
  getChannelId(): string;
  setChannelId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetChannelDataRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetChannelDataRequest): GetChannelDataRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetChannelDataRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetChannelDataRequest;
  static deserializeBinaryFromReader(message: GetChannelDataRequest, reader: jspb.BinaryReader): GetChannelDataRequest;
}

export namespace GetChannelDataRequest {
  export type AsObject = {
    channelId: string,
  }
}

export class GetChannelDataResponse extends jspb.Message {
  getState(): GetChannelDataResponse.StateMap[keyof GetChannelDataResponse.StateMap];
  setState(value: GetChannelDataResponse.StateMap[keyof GetChannelDataResponse.StateMap]): void;

  getBalance(): string;
  setBalance(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetChannelDataResponse.AsObject;
  static toObject(includeInstance: boolean, msg: GetChannelDataResponse): GetChannelDataResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetChannelDataResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetChannelDataResponse;
  static deserializeBinaryFromReader(message: GetChannelDataResponse, reader: jspb.BinaryReader): GetChannelDataResponse;
}

export namespace GetChannelDataResponse {
  export type AsObject = {
    state: GetChannelDataResponse.StateMap[keyof GetChannelDataResponse.StateMap],
    balance: string,
  }

  export interface StateMap {
    UNKNOWN: 0;
    UNINITIALISED: 1;
    FUNDED: 2;
    OPEN: 3;
    PENDING: 4;
  }

  export const State: StateMap;
}

export class OpenChannelRequest extends jspb.Message {
  getPeerId(): string;
  setPeerId(value: string): void;

  getAmount(): string;
  setAmount(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OpenChannelRequest.AsObject;
  static toObject(includeInstance: boolean, msg: OpenChannelRequest): OpenChannelRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: OpenChannelRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OpenChannelRequest;
  static deserializeBinaryFromReader(message: OpenChannelRequest, reader: jspb.BinaryReader): OpenChannelRequest;
}

export namespace OpenChannelRequest {
  export type AsObject = {
    peerId: string,
    amount: string,
  }
}

export class OpenChannelResponse extends jspb.Message {
  getChannelId(): string;
  setChannelId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OpenChannelResponse.AsObject;
  static toObject(includeInstance: boolean, msg: OpenChannelResponse): OpenChannelResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: OpenChannelResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OpenChannelResponse;
  static deserializeBinaryFromReader(message: OpenChannelResponse, reader: jspb.BinaryReader): OpenChannelResponse;
}

export namespace OpenChannelResponse {
  export type AsObject = {
    channelId: string,
  }
}

export class CloseChannelRequest extends jspb.Message {
  getChannelId(): string;
  setChannelId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CloseChannelRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CloseChannelRequest): CloseChannelRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CloseChannelRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CloseChannelRequest;
  static deserializeBinaryFromReader(message: CloseChannelRequest, reader: jspb.BinaryReader): CloseChannelRequest;
}

export namespace CloseChannelRequest {
  export type AsObject = {
    channelId: string,
  }
}

export class CloseChannelResponse extends jspb.Message {
  getChannelId(): string;
  setChannelId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CloseChannelResponse.AsObject;
  static toObject(includeInstance: boolean, msg: CloseChannelResponse): CloseChannelResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CloseChannelResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CloseChannelResponse;
  static deserializeBinaryFromReader(message: CloseChannelResponse, reader: jspb.BinaryReader): CloseChannelResponse;
}

export namespace CloseChannelResponse {
  export type AsObject = {
    channelId: string,
  }
}

