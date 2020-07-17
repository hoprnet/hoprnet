import { join } from 'path'

export const HOPR_PROTOS_DIR = join(__dirname, '..', 'node_modules', '@hoprnet/hopr-protos')
export const HOPR_PROTOS_FOLDER_DIR = join(HOPR_PROTOS_DIR, 'protos')
export const PROTO_PACKAGES = [
  'status',
  'version',
  'shutdown',
  'ping',
  'address',
  'balance',
  'channels',
  'send',
  'listen',
]
export const PROTO_FILES = PROTO_PACKAGES.map((pkg) => `${pkg}.proto`)
