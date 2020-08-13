const path = require('path')
const glob = require('glob')

const ROOT = path.join(__dirname, '..')
const PROTOC_GEN_TS = path.join(ROOT, 'node_modules', '.bin', 'protoc-gen-ts')
const PROTOC_GEN_GRPC = path.join(ROOT, 'node_modules', '.bin', 'grpc_tools_node_protoc_plugin')
const PROTOC_GEN_GRPC_WEB = path.join(ROOT, 'node_modules', '.bin', 'protoc-gen-grpc-web')
const INPUT_DIR = path.join(ROOT, 'protos')
const protos = glob.sync(path.join(INPUT_DIR, '**', '*.proto')).map((dir) => path.join('protos', path.basename(dir)))

module.exports = {
  ROOT,
  PROTOC_GEN_TS,
  PROTOC_GEN_GRPC,
  PROTOC_GEN_GRPC_WEB,
  INPUT_DIR,
  protos,
}
