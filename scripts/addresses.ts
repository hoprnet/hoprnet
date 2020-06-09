export type Networks = 'mainnet' | 'morden' | 'ropsten' | 'rinkeby' | 'goerli' | 'kovan' | 'private'

export const HOPR_TOKEN: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0x0575C2D12E15D11a088C9012449d79caE050996c',
  private: '0x66DB78F4ADD912a6Cb92b672Dfa09028ecc3085E',
}

export const HOPR_CHANNELS: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0x6e3FC1cA7935ba169d4fa10205B72A7300614c48',
  private: '0x902602174a9cEb452f60c09043BE5EBC52096200',
}

export const HOPR_MINTER: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: undefined,
  private: '0x0a67180CF519aDF27f1FD32F7255bBa00B536FC6',
}

export const HOPR_FAUCET: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0xa75159bC8936d471d8E272a358d840a899447dCa',
  private: '0x6c97048D67c39ADCe38bbB228fc1bA415fc8f096',
}
