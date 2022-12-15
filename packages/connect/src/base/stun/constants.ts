// STUN server constants
export const isStunRequest = 0x0000
// const isStunIndication = 0x0010
export const isStunSuccessResponse = 0x0100
export const isStunErrorResponse = 0x0110
export const kStunTypeMask = 0x0110

// Used to track process of a RFC 5780 STUN request
export enum STUN_QUERY_STATE {
  SEARCHING_RFC_5780_STUN_SERVER,
  CHECKING_PORT_MAPPING
}

export enum STUN_EXPOSED_CHECK_RESPOSE {
  EXPOSED,
  NOT_EXPOSED,
  UNKNOWN
}

export function exposedResponseToString(response: STUN_EXPOSED_CHECK_RESPOSE) {
  switch (response) {
    case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
      return 'exposed'
    case STUN_EXPOSED_CHECK_RESPOSE.NOT_EXPOSED:
      return 'not exposed'
    case STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN:
      return 'unknown'
  }
}
