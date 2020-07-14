const path = require('path')
const { execFile } = require('child_process')
const { ROOT, PROTOC_GEN_TS, protos } = require('./utils')

const OUTPUT_DIR = path.join(ROOT, 'web-src')

const args = [
  'grpc_tools_node_protoc',
  `--proto_path=protos`,
  `--plugin=protoc-gen-ts=${PROTOC_GEN_TS}`,
  `--js_out=import_style=commonjs,binary:${OUTPUT_DIR}`,
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
