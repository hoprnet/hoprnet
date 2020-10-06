# Protocol Documentation

<a name="top"></a>

## Table of Contents

- [address.proto](#address.proto)

  - [GetHoprAddressRequest](#address.GetHoprAddressRequest)
  - [GetHoprAddressResponse](#address.GetHoprAddressResponse)
  - [GetNativeAddressRequest](#address.GetNativeAddressRequest)
  - [GetNativeAddressResponse](#address.GetNativeAddressResponse)

  - [Address](#address.Address)

- [balance.proto](#balance.proto)

  - [GetHoprBalanceRequest](#balance.GetHoprBalanceRequest)
  - [GetHoprBalanceResponse](#balance.GetHoprBalanceResponse)
  - [GetNativeBalanceRequest](#balance.GetNativeBalanceRequest)
  - [GetNativeBalanceResponse](#balance.GetNativeBalanceResponse)

  - [Balance](#balance.Balance)

- [channels.proto](#channels.proto)

  - [CloseChannelRequest](#channels.CloseChannelRequest)
  - [CloseChannelResponse](#channels.CloseChannelResponse)
  - [GetChannelDataRequest](#channels.GetChannelDataRequest)
  - [GetChannelDataResponse](#channels.GetChannelDataResponse)
  - [GetChannelsRequest](#channels.GetChannelsRequest)
  - [GetChannelsResponse](#channels.GetChannelsResponse)
  - [OpenChannelRequest](#channels.OpenChannelRequest)
  - [OpenChannelResponse](#channels.OpenChannelResponse)

  - [GetChannelDataResponse.State](#channels.GetChannelDataResponse.State)

  - [Channels](#channels.Channels)

- [listen.proto](#listen.proto)

  - [ListenRequest](#listen.ListenRequest)
  - [ListenResponse](#listen.ListenResponse)

  - [Listen](#listen.Listen)

- [ping.proto](#ping.proto)

  - [PingRequest](#ping.PingRequest)
  - [PingResponse](#ping.PingResponse)

  - [Ping](#ping.Ping)

- [send.proto](#send.proto)

  - [SendRequest](#send.SendRequest)
  - [SendResponse](#send.SendResponse)

  - [Send](#send.Send)

- [settings.proto](#settings.proto)

  - [UpdateSettingsRequest](#ping.UpdateSettingsRequest)
  - [UpdateSettingsResponse](#ping.UpdateSettingsResponse)

  - [Settings](#ping.Settings)

- [shutdown.proto](#shutdown.proto)

  - [ShutdownRequest](#shutdown.ShutdownRequest)
  - [ShutdownResponse](#shutdown.ShutdownResponse)

  - [Shutdown](#shutdown.Shutdown)

- [status.proto](#status.proto)

  - [StatusRequest](#status.StatusRequest)
  - [StatusResponse](#status.StatusResponse)

  - [Status](#status.Status)

- [version.proto](#version.proto)

  - [VersionRequest](#version.VersionRequest)
  - [VersionResponse](#version.VersionResponse)
  - [VersionResponse.ComponentsVersionEntry](#version.VersionResponse.ComponentsVersionEntry)

  - [Version](#version.Version)

- [Scalar Value Types](#scalar-value-types)

<a name="address.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## address.proto

<a name="address.GetHoprAddressRequest"></a>

### GetHoprAddressRequest

<a name="address.GetHoprAddressResponse"></a>

### GetHoprAddressResponse

| Field   | Type              | Label | Description |
| ------- | ----------------- | ----- | ----------- |
| address | [string](#string) |       |             |

<a name="address.GetNativeAddressRequest"></a>

### GetNativeAddressRequest

<a name="address.GetNativeAddressResponse"></a>

### GetNativeAddressResponse

| Field   | Type              | Label | Description |
| ------- | ----------------- | ----- | ----------- |
| address | [string](#string) |       |             |

<a name="address.Address"></a>

### Address

| Method Name      | Request Type                                                | Response Type                                                 | Description               |
| ---------------- | ----------------------------------------------------------- | ------------------------------------------------------------- | ------------------------- |
| GetNativeAddress | [GetNativeAddressRequest](#address.GetNativeAddressRequest) | [GetNativeAddressResponse](#address.GetNativeAddressResponse) | example: ethereum address |
| GetHoprAddress   | [GetHoprAddressRequest](#address.GetHoprAddressRequest)     | [GetHoprAddressResponse](#address.GetHoprAddressResponse)     |                           |

<a name="balance.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## balance.proto

<a name="balance.GetHoprBalanceRequest"></a>

### GetHoprBalanceRequest

<a name="balance.GetHoprBalanceResponse"></a>

### GetHoprBalanceResponse

| Field  | Type              | Label | Description |
| ------ | ----------------- | ----- | ----------- |
| amount | [string](#string) |       |             |

<a name="balance.GetNativeBalanceRequest"></a>

### GetNativeBalanceRequest

<a name="balance.GetNativeBalanceResponse"></a>

### GetNativeBalanceResponse

| Field  | Type              | Label | Description |
| ------ | ----------------- | ----- | ----------- |
| amount | [string](#string) |       |             |

<a name="balance.Balance"></a>

### Balance

| Method Name      | Request Type                                                | Response Type                                                 | Description    |
| ---------------- | ----------------------------------------------------------- | ------------------------------------------------------------- | -------------- |
| GetNativeBalance | [GetNativeBalanceRequest](#balance.GetNativeBalanceRequest) | [GetNativeBalanceResponse](#balance.GetNativeBalanceResponse) | example: ETHER |
| GetHoprBalance   | [GetHoprBalanceRequest](#balance.GetHoprBalanceRequest)     | [GetHoprBalanceResponse](#balance.GetHoprBalanceResponse)     |                |

<a name="channels.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## channels.proto

<a name="channels.CloseChannelRequest"></a>

### CloseChannelRequest

| Field      | Type              | Label | Description |
| ---------- | ----------------- | ----- | ----------- |
| channel_id | [string](#string) |       |             |

<a name="channels.CloseChannelResponse"></a>

### CloseChannelResponse

| Field      | Type              | Label | Description |
| ---------- | ----------------- | ----- | ----------- |
| channel_id | [string](#string) |       |             |

<a name="channels.GetChannelDataRequest"></a>

### GetChannelDataRequest

| Field      | Type              | Label | Description |
| ---------- | ----------------- | ----- | ----------- |
| channel_id | [string](#string) |       |             |

<a name="channels.GetChannelDataResponse"></a>

### GetChannelDataResponse

| Field   | Type                                                                   | Label | Description |
| ------- | ---------------------------------------------------------------------- | ----- | ----------- |
| state   | [GetChannelDataResponse.State](#channels.GetChannelDataResponse.State) |       |             |
| balance | [string](#string)                                                      |       |             |

<a name="channels.GetChannelsRequest"></a>

### GetChannelsRequest

<a name="channels.GetChannelsResponse"></a>

### GetChannelsResponse

| Field    | Type              | Label    | Description |
| -------- | ----------------- | -------- | ----------- |
| channels | [string](#string) | repeated |             |

<a name="channels.OpenChannelRequest"></a>

### OpenChannelRequest

| Field   | Type              | Label | Description |
| ------- | ----------------- | ----- | ----------- |
| peer_id | [string](#string) |       |             |
| amount  | [string](#string) |       |             |

<a name="channels.OpenChannelResponse"></a>

### OpenChannelResponse

| Field      | Type              | Label | Description |
| ---------- | ----------------- | ----- | ----------- |
| channel_id | [string](#string) |       |             |

<a name="channels.GetChannelDataResponse.State"></a>

### GetChannelDataResponse.State

| Name          | Number | Description |
| ------------- | ------ | ----------- |
| UNKNOWN       | 0      |             |
| UNINITIALISED | 1      |             |
| FUNDED        | 2      |             |
| OPEN          | 3      |             |
| PENDING       | 4      |             |

<a name="channels.Channels"></a>

### Channels

| Method Name    | Request Type                                             | Response Type                                              | Description                                                                        |
| -------------- | -------------------------------------------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| GetChannels    | [GetChannelsRequest](#channels.GetChannelsRequest)       | [GetChannelsResponse](#channels.GetChannelsResponse)       |                                                                                    |
| GetChannelData | [GetChannelDataRequest](#channels.GetChannelDataRequest) | [GetChannelDataResponse](#channels.GetChannelDataResponse) | unable to name this &#39;GetChannel&#39; because it&#39;s already used by the stub |
| OpenChannel    | [OpenChannelRequest](#channels.OpenChannelRequest)       | [OpenChannelResponse](#channels.OpenChannelResponse)       |                                                                                    |
| CloseChannel   | [CloseChannelRequest](#channels.CloseChannelRequest)     | [CloseChannelResponse](#channels.CloseChannelResponse)     |                                                                                    |

<a name="listen.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## listen.proto

<a name="listen.ListenRequest"></a>

### ListenRequest

| Field   | Type              | Label | Description |
| ------- | ----------------- | ----- | ----------- |
| peer_id | [string](#string) |       |             |

<a name="listen.ListenResponse"></a>

### ListenResponse

| Field   | Type            | Label | Description |
| ------- | --------------- | ----- | ----------- |
| payload | [bytes](#bytes) |       |             |

<a name="listen.Listen"></a>

### Listen

| Method Name | Request Type                           | Response Type                                   | Description |
| ----------- | -------------------------------------- | ----------------------------------------------- | ----------- |
| Listen      | [ListenRequest](#listen.ListenRequest) | [ListenResponse](#listen.ListenResponse) stream |             |

<a name="ping.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## ping.proto

<a name="ping.PingRequest"></a>

### PingRequest

| Field   | Type              | Label | Description |
| ------- | ----------------- | ----- | ----------- |
| peer_id | [string](#string) |       |             |

<a name="ping.PingResponse"></a>

### PingResponse

| Field   | Type            | Label | Description  |
| ------- | --------------- | ----- | ------------ |
| latency | [int32](#int32) |       | milliseconds |

<a name="ping.Ping"></a>

### Ping

| Method Name | Request Type                     | Response Type                      | Description |
| ----------- | -------------------------------- | ---------------------------------- | ----------- |
| GetPing     | [PingRequest](#ping.PingRequest) | [PingResponse](#ping.PingResponse) |             |

<a name="send.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## send.proto

<a name="send.SendRequest"></a>

### SendRequest

| Field   | Type              | Label | Description |
| ------- | ----------------- | ----- | ----------- |
| peer_id | [string](#string) |       |             |
| payload | [bytes](#bytes)   |       |             |

<a name="send.SendResponse"></a>

### SendResponse

| Field                 | Type              | Label    | Description |
| --------------------- | ----------------- | -------- | ----------- |
| intermediate_peer_ids | [string](#string) | repeated |             |

<a name="send.Send"></a>

### Send

| Method Name | Request Type                     | Response Type                      | Description |
| ----------- | -------------------------------- | ---------------------------------- | ----------- |
| Send        | [SendRequest](#send.SendRequest) | [SendResponse](#send.SendResponse) |             |

<a name="settings.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## settings.proto

<a name="ping.UpdateSettingsRequest"></a>

### UpdateSettingsRequest

| Field                  | Type              | Label    | Description |
| ---------------------- | ----------------- | -------- | ----------- |
| is_using_cover_traffic | [bool](#bool)     |          |             |
| bootstrap_servers      | [string](#string) | repeated |             |

<a name="ping.UpdateSettingsResponse"></a>

### UpdateSettingsResponse

| Field   | Type            | Label | Description  |
| ------- | --------------- | ----- | ------------ |
| latency | [int32](#int32) |       | milliseconds |

<a name="ping.Settings"></a>

### Settings

| Method Name    | Request Type                                         | Response Type                                          | Description                                           |
| -------------- | ---------------------------------------------------- | ------------------------------------------------------ | ----------------------------------------------------- |
| UpdateSettings | [UpdateSettingsRequest](#ping.UpdateSettingsRequest) | [UpdateSettingsResponse](#ping.UpdateSettingsResponse) | update setting on the fly without requiring a restart |

<a name="shutdown.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## shutdown.proto

<a name="shutdown.ShutdownRequest"></a>

### ShutdownRequest

<a name="shutdown.ShutdownResponse"></a>

### ShutdownResponse

| Field     | Type            | Label | Description |
| --------- | --------------- | ----- | ----------- |
| timestamp | [int32](#int32) |       | seconds     |

<a name="shutdown.Shutdown"></a>

### Shutdown

| Method Name | Request Type                                 | Response Type                                  | Description |
| ----------- | -------------------------------------------- | ---------------------------------------------- | ----------- |
| Shutdown    | [ShutdownRequest](#shutdown.ShutdownRequest) | [ShutdownResponse](#shutdown.ShutdownResponse) |             |

<a name="status.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## status.proto

<a name="status.StatusRequest"></a>

### StatusRequest

<a name="status.StatusResponse"></a>

### StatusResponse

| Field           | Type              | Label    | Description |
| --------------- | ----------------- | -------- | ----------- |
| id              | [string](#string) |          |             |
| multi_addresses | [string](#string) | repeated |             |
| connected_nodes | [int32](#int32)   |          |             |

<a name="status.Status"></a>

### Status

| Method Name | Request Type                           | Response Type                            | Description |
| ----------- | -------------------------------------- | ---------------------------------------- | ----------- |
| GetStatus   | [StatusRequest](#status.StatusRequest) | [StatusResponse](#status.StatusResponse) |             |

<a name="version.proto"></a>

<p align="right"><a href="#top">Top</a></p>

## version.proto

<a name="version.VersionRequest"></a>

### VersionRequest

<a name="version.VersionResponse"></a>

### VersionResponse

| Field              | Type                                                                                      | Label    | Description |
| ------------------ | ----------------------------------------------------------------------------------------- | -------- | ----------- |
| version            | [string](#string)                                                                         |          |             |
| components_version | [VersionResponse.ComponentsVersionEntry](#version.VersionResponse.ComponentsVersionEntry) | repeated |             |

<a name="version.VersionResponse.ComponentsVersionEntry"></a>

### VersionResponse.ComponentsVersionEntry

| Field | Type              | Label | Description |
| ----- | ----------------- | ----- | ----------- |
| key   | [string](#string) |       |             |
| value | [string](#string) |       |             |

<a name="version.Version"></a>

### Version

| Method Name | Request Type                              | Response Type                               | Description |
| ----------- | ----------------------------------------- | ------------------------------------------- | ----------- |
| GetVersion  | [VersionRequest](#version.VersionRequest) | [VersionResponse](#version.VersionResponse) |             |

## Scalar Value Types

| .proto Type                    | Notes                                                                                                                                           | C++    | Java       | Python      | Go      | C#         | PHP            | Ruby                           |
| ------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------ | ---------- | ----------- | ------- | ---------- | -------------- | ------------------------------ |
| <a name="double" /> double     |                                                                                                                                                 | double | double     | float       | float64 | double     | float          | Float                          |
| <a name="float" /> float       |                                                                                                                                                 | float  | float      | float       | float32 | float      | float          | Float                          |
| <a name="int32" /> int32       | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint32 instead. | int32  | int        | int         | int32   | int        | integer        | Bignum or Fixnum (as required) |
| <a name="int64" /> int64       | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint64 instead. | int64  | long       | int/long    | int64   | long       | integer/string | Bignum                         |
| <a name="uint32" /> uint32     | Uses variable-length encoding.                                                                                                                  | uint32 | int        | int/long    | uint32  | uint       | integer        | Bignum or Fixnum (as required) |
| <a name="uint64" /> uint64     | Uses variable-length encoding.                                                                                                                  | uint64 | long       | int/long    | uint64  | ulong      | integer/string | Bignum or Fixnum (as required) |
| <a name="sint32" /> sint32     | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int32s.                            | int32  | int        | int         | int32   | int        | integer        | Bignum or Fixnum (as required) |
| <a name="sint64" /> sint64     | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int64s.                            | int64  | long       | int/long    | int64   | long       | integer/string | Bignum                         |
| <a name="fixed32" /> fixed32   | Always four bytes. More efficient than uint32 if values are often greater than 2^28.                                                            | uint32 | int        | int         | uint32  | uint       | integer        | Bignum or Fixnum (as required) |
| <a name="fixed64" /> fixed64   | Always eight bytes. More efficient than uint64 if values are often greater than 2^56.                                                           | uint64 | long       | int/long    | uint64  | ulong      | integer/string | Bignum                         |
| <a name="sfixed32" /> sfixed32 | Always four bytes.                                                                                                                              | int32  | int        | int         | int32   | int        | integer        | Bignum or Fixnum (as required) |
| <a name="sfixed64" /> sfixed64 | Always eight bytes.                                                                                                                             | int64  | long       | int/long    | int64   | long       | integer/string | Bignum                         |
| <a name="bool" /> bool         |                                                                                                                                                 | bool   | boolean    | boolean     | bool    | bool       | boolean        | TrueClass/FalseClass           |
| <a name="string" /> string     | A string must always contain UTF-8 encoded or 7-bit ASCII text.                                                                                 | string | String     | str/unicode | string  | string     | string         | String (UTF-8)                 |
| <a name="bytes" /> bytes       | May contain any arbitrary sequence of bytes.                                                                                                    | string | ByteString | str         | []byte  | ByteString | string         | String (ASCII-8BIT)            |
