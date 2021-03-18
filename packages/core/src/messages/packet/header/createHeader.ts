import secp256k1 from 'secp256k1'
import { randomBytes } from 'crypto'
import { u8aXOR, u8aConcat, PRG } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '../../../constants'

import {
  Header,
  BETA_LENGTH,
  deriveBlinding,
  derivePRGParameters,
  deriveTicketKey,
  deriveTicketKeyBlinding,
  deriveTicketLastKey,
  createMAC
} from './index'

import type { Types } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'

import {
  PRIVATE_KEY_LENGTH,
  PER_HOP_SIZE,
  DESINATION_SIZE,
  IDENTIFIER_SIZE,
  ADDRESS_SIZE,
  MAC_SIZE,
  LAST_HOP_SIZE,
  KEY_LENGTH
} from './parameters'

