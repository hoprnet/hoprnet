export const vmErrorMessage = (error: string) => {
  return `VM Exception while processing transaction: revert ${error}`
}
