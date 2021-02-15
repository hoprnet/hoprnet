export type HoprDistributorParams = {
  network: string
  startTime: string // seconds
  maxMintAmount: string // HOPRli
  multisig: string // address
}

export type Schedule = {
  name: string
  durations: string[] // seconds
  percents: string[] // 1 * multiplier used in contract
}

export type Allocations = {
  name: string
  accounts: string[] // address
  amounts: string[] // HOPRli
}
