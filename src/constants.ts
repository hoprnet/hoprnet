import { Constants as IConstants } from '@hoprnet/hopr-core-connector-interface'

export default class implements IConstants {
  HASH_LENGTH: 32
  SIGNATURE_LENGTH: 65
  COMPRESSED_PUBLIC_KEY_LENGTH: 33
  ETHEUREUM_ADDRESS_LENGTH: 32
  SIGNATURE_RECOVERY_LENGTH: 1
}
