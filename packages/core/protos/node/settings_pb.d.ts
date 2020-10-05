// package: ping
// file: settings.proto

import * as jspb from "google-protobuf";

export class UpdateSettingsRequest extends jspb.Message {
  getIsUsingCoverTraffic(): boolean;
  setIsUsingCoverTraffic(value: boolean): void;

  clearBootstrapServersList(): void;
  getBootstrapServersList(): Array<string>;
  setBootstrapServersList(value: Array<string>): void;
  addBootstrapServers(value: string, index?: number): string;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSettingsRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSettingsRequest): UpdateSettingsRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: UpdateSettingsRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSettingsRequest;
  static deserializeBinaryFromReader(message: UpdateSettingsRequest, reader: jspb.BinaryReader): UpdateSettingsRequest;
}

export namespace UpdateSettingsRequest {
  export type AsObject = {
    isUsingCoverTraffic: boolean,
    bootstrapServersList: Array<string>,
  }
}

export class UpdateSettingsResponse extends jspb.Message {
  getLatency(): number;
  setLatency(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UpdateSettingsResponse.AsObject;
  static toObject(includeInstance: boolean, msg: UpdateSettingsResponse): UpdateSettingsResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: UpdateSettingsResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UpdateSettingsResponse;
  static deserializeBinaryFromReader(message: UpdateSettingsResponse, reader: jspb.BinaryReader): UpdateSettingsResponse;
}

export namespace UpdateSettingsResponse {
  export type AsObject = {
    latency: number,
  }
}

