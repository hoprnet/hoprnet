# HOPR

HOPR is a privacy-preserving messaging **protocol** which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.

## Protos

See an overview of the protos [here](./doc/protos.md)

## Testing

Testing is done by trying to generate proto stubs for node and web, if building fails then it means something is wrong with our protos.
Ideally, we can improve this to also include linting, etc.

## Notes

### Libraries

- [grpc-js](https://github.com/grpc/grpc-node/tree/master/packages/grpc-js)
  - our is it to use JS implementantion since the native one is [deprecated](https://grpc.io/blog/grpc-js-1.0/#should-i-use-grpcgrpc-js-or-grpc), see [comparison](https://github.com/grpc/grpc-node/blob/master/PACKAGE-COMPARISON.md). Unfortunately at the moment we use grpc-native since nestjs doesn't support grpc-js
- [protoc-gen-doc](https://github.com/pseudomuto/protoc-gen-doc)

### General Guidelines

- [Status codes and their use in gRPC](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md)
- [HTTP to gRPC Status Code Mapping](https://github.com/grpc/grpc/blob/master/doc/http-grpc-status-mapping.md)
- [gRPC Errors - A handy guide to gRPC errors.](https://github.com/avinassh/grpc-errors)
- [GCP API Design Guide](https://cloud.google.com/apis/design)

### Testing Guidelines

- [How to write unit tests for gRPC C client.](https://github.com/grpc/grpc/blob/master/doc/unit_testing.md)

### Extra

- [GRPC Health Checking Protocol](https://github.com/grpc/grpc/blob/master/doc/health-checking.md)
- [GRPC Technical Documentation](https://github.com/grpc/grpc/tree/master/doc)
- [Google's protos](https://github.com/googleapis/googleapis/tree/master/google)
- [buf](https://buf.build/)
- [awesome-grpc](https://github.com/grpc-ecosystem/awesome-grpc)
