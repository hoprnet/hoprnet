export type Networks = 'mainnet' | 'morden' | 'ropsten' | 'rinkeby' | 'goerli' | 'kovan' | 'private'

export const HOPR_TOKEN: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0xedEaeA1BaAc154D47dd20eC0b8952dB0a4224dd0',
  private: '0x66DB78F4ADD912a6Cb92b672Dfa09028ecc3085E',
}

export const HOPR_CHANNELS: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0xFA4530CAEF7f19AEa4c3a7A1880AB2edB5803f31',
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
  kovan: '0x827557073244d7d73d73d3Bf3cD3c7C546867f28',
  private: '0x6c97048D67c39ADCe38bbB228fc1bA415fc8f096',
}
