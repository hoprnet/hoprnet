export const isError = (error: any): error is Error => {
  return error instanceof Error
}

export const _createTestState = () => ({ aliases: new Map(), includeRecipient: false })
