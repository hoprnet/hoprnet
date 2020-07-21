#!/bin/bash

parent_path=$( cd "$(dirname "${BASH_SOURCE[1]}")" ; pwd -P )

docker run --rm \
  -v "$parent_path/docs":/out \
  -v "$parent_path/protos":/protos \
  pseudomuto/protoc-gen-doc --doc_opt=markdown,protos.md