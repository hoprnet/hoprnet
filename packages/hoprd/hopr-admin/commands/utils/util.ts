
// HOPR -> weiHOPR
export const hoprToWei = (value: string) => Math.floor(Number((BigInt(value) * 10n ** 18n).toString()))

// weiHOPR -> HOPR
// export const weiToHopr = ()