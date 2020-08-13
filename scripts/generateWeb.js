const path = require('path')
const { execFile } = require('child_process')
const { ROOT, PROTOC_GEN_GRPC_WEB, protos } = require('./utils')

const OUTPUT_DIR = path.join(ROOT, 'web')

const args = [
  'grpc_tools_node_protoc',
  `--proto_path=protos`,
  `--plugin=protoc-gen-grpc-web=${PROTOC_GEN_GRPC_WEB}`,
  `--js_out=import_style=commonjs,binary:${OUTPUT_DIR}`,
  `--grpc-web_out=import_style=commonjs+dts,mode=grpcwebtext:${OUTPUT_DIR}`,
  ...protos,
]

const child_process = execFile('npx', args, function (error, stdout, stderr) {
  if (error) {
    throw error
  }
})

child_process.stdout.pipe(process.stdout)
child_process.stderr.pipe(process.stderr)
