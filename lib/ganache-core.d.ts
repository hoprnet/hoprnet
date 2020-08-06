// temporary workaround to https://github.com/trufflesuite/ganache-core/issues/465

declare module 'web3/providers' {
  type Web3Provider = any
  type Provider = any
}
