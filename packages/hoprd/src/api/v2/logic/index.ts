import { APIv2State } from "../../v2"

export const isError = (error: any): error is Error => {
  return error instanceof Error
}

export const _createTestState = (): APIv2State => ({ aliases: new Map(), settings: { includeRecipient: false, strategy: "passive" } })
