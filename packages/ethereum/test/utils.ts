/**
 * Depending on what network tests are run, the error output
 * may vary. This utility prefixes the error to it matches
 * with hardhat's network.
 * @param error
 * @returns error prefixed by network's message
 */
export const vmErrorMessage = (error: string) => {
  return `VM Exception while processing transaction: revert ${error}`
}
