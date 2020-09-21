import { join, dirname } from 'path'

export const HOPR_CORE_DIR = dirname(require.resolve('@hoprnet/hopr-core'))
export const HOPR_PROTOS_DIR = dirname(require.resolve('@hoprnet/hopr-protos'))
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
  'withdraw',
]
export const PROTO_FILES = PROTO_PACKAGES.map((pkg) => `${pkg}.proto`)
