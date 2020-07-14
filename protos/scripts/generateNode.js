const path = require('path')
const { execFile } = require('child_process')
const { ROOT, PROTOC_GEN_TS, PROTOC_GEN_GRPC, protos } = require('./utils')

const OUTPUT_DIR = path.join(ROOT, 'node')

const args = [
  'grpc_tools_node_protoc',
  `--proto_path=protos`,
  `--plugin=protoc-gen-grpc=${PROTOC_GEN_GRPC}`,
  `--plugin=protoc-gen-ts=${PROTOC_GEN_TS}`,
  `--js_out=import_style=commonjs,binary:${OUTPUT_DIR}`,
  // `--grpc_out=grpc_js:${OUTPUT_DIR}`, // @grpc/grpc-js output
  `--grpc_out=${OUTPUT_DIR}`,
  `--ts_out=service=grpc-node:${OUTPUT_DIR}`,
  ...protos,
]

const child_process = execFile('npx', args, function (error, stdout, stderr) {
  if (error) {
    throw error
  }
})

child_process.stdout.pipe(process.stdout)
child_process.stderr.pipe(process.stderr)
