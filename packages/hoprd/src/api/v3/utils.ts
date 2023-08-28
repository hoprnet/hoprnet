/**
 * At the moment, using our own custom codes
 * and validations in the possibilty we want to
 * reuse the code for commands, will change if said
 * otherwise.
 */
export enum STATUS_CODES {
  // invalid inputs
  INVALID_INPUT = 'INVALID_INPUT',
  INVALID_PEERID = 'INVALID_PEERID',
  INVALID_CHANNELID = 'INVALID_CHANNELID',
  INVALID_CURRENCY = 'INVALID_CURRENCY',
  INVALID_AMOUNT = 'INVALID_AMOUNT',
  INVALID_ADDRESS = 'INVALID_ADDRESS',
  INVALID_SETTING = 'INVALID_SETTING',
  INVALID_SETTING_VALUE = 'INVALID_SETTING_VALUE',
  INVALID_QUALITY = 'INVALID_QUALITY',
  INVALID_TOKEN_LIFETIME = 'INVALID_TOKEN_LIFETIME',
  INVALID_TOKEN_CAPABILITIES = 'INVALID_TOKEN_CAPABILITIES',
  INVALID_TOKEN_DESCRIPTION = 'INVALID_TOKEN_DESCRIPTION',
  // protocol
  PEERID_NOT_FOUND = 'PEERID_NOT_FOUND',
  CHANNEL_NOT_FOUND = 'CHANNEL_NOT_FOUND',
  TICKETS_NOT_FOUND = 'TICKETS_NOT_FOUND',
  NOT_ENOUGH_BALANCE = 'NOT_ENOUGH_BALANCE',
  NOT_ENOUGH_ALLOWANCE = 'NOT_ENOUGH_ALLOWANCE',
  CHANNEL_ALREADY_OPEN = 'CHANNEL_ALREADY_OPEN',
  TIMEOUT = 'TIMEOUT',
  // other
  UNKNOWN_FAILURE = 'UNKNOWN_FAILURE',
  FORBIDDEN = 'FORBIDDEN',
  UNAUTHORIZED = 'UNAUTHORIZED',
  // initiate/close incoming channel is not supported in monte_rosa
  UNSUPPORTED_FEATURE = 'UNSUPPORTED_FEATURE'
}

/**
 * Default responses when for documenting websocket endpoints.
 */
export const WS_DEFAULT_RESPONSES: Record<string, { description: string }> = {
  '101': {
    description: 'Switching protocols'
  },
  '401': {
    description: 'Unauthorized'
  },
  '404': {
    description: 'Not found'
  }
}

/**
 * Generate a websocket endpoint description suffixed with general security data.
 * @param summary Short summary to prefix the endpoint's description.
 * @param path Path of the endpoint after `/api/v2`.
 * @returns endpoint's description
 */
export const generateWsApiDescription = (summary: string, path: string): string => {
  return `${summary} Authentication (if enabled) is done via either passing an \`apiToken\` parameter in the url or cookie \`X-Auth-Token\`. Connect to the endpoint by using a WS client. No preview available. Example: \`ws://127.0.0.1:3001/api/v2${path}/?apiToken=myApiToken\``
}
