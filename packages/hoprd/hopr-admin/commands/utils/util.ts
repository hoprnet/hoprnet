// export enum ChannelStatus {
//   Closed = "Closed",
//   WaitingForCommitment = "WaitingForCommitment",
//   Open = "Open",
//   PendingToClose = "PendingToClose"
// }

export enum BalanceDecimals {
  Native = 18,
  Balance = 18
}

export enum BalanceSymbols {
  Native = 'xDAI',
  Balance = 'txHOPR'
}

// HOPR -> weiHOPR
// export const hoprToWei = (value: string) => Math.floor(Number((BigInt(value) * 10n ** 18n).toString()))

// weiHOPR -> HOPR
// export const weiToHopr = ()