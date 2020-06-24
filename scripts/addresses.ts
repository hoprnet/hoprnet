export type Networks = 'mainnet' | 'morden' | 'ropsten' | 'rinkeby' | 'goerli' | 'kovan' | 'private'

export const HOPR_TOKEN: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0xDD78859E6714D045a31Caa0075C1523f6E08fFe1',
  private: '0x66DB78F4ADD912a6Cb92b672Dfa09028ecc3085E',
}

export const HOPR_CHANNELS: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0x6eCe0EC9E5F408e664ACc397A8Ac7241841c6658',
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
  kovan: '0x034869aaF67F09296303D2d42dceEc53F4F04533',
  private: '0x6c97048D67c39ADCe38bbB228fc1bA415fc8f096',
}
