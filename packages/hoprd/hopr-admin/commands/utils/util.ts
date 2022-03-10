export enum ChannelStatus {
  Closed = 0,
  WaitingForCommitment = 1,
  Open = 2,
  PendingToClose = 3
}

export enum BalanceDecimals {
  Native = 18,
  Balance = 18
}

export enum BalanceSymbols {
  Native = 'xDAI',
  Balance = 'txHOPR'
}

// HOPR -> weiHOPR
export const hoprToWei = (value: string) => Math.floor(Number((BigInt(value) * 10n ** 18n).toString()))

// weiHOPR -> HOPR
// export const weiToHopr = ()