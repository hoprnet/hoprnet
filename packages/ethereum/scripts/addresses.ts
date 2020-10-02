export type Networks = 'mainnet' | 'morden' | 'ropsten' | 'rinkeby' | 'goerli' | 'kovan' | 'private' | 'solkol' | 'xdai'

export const HOPR_TOKEN: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0xDD78859E6714D045a31Caa0075C1523f6E08fFe1',
  private: '0x66DB78F4ADD912a6Cb92b672Dfa09028ecc3085E',
  solkol: undefined,
  xdai: '0x12481c3Ed97b32D94E71C2039DBC44432ADD39a0',
}

export const HOPR_CHANNELS: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0x6eCe0EC9E5F408e664ACc397A8Ac7241841c6658',
  private: '0x0a67180CF519aDF27f1FD32F7255bBa00B536FC6',
  solkol: undefined,
  xdai: '0x75E35d2Db3193670C8a55308180C2c3aD63A6ab5',
}

export const HOPR_MINTER: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: undefined,
  private: '0x6c97048D67c39ADCe38bbB228fc1bA415fc8f096',
  solkol: undefined,
  xdai: '',
}

export const HOPR_FAUCET: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: undefined,
  goerli: undefined,
  kovan: '0x034869aaF67F09296303D2d42dceEc53F4F04533',
  private: '0x2E2c8a6710cb5168ec3362a9c1280E2A1FBf0B5E',
  solkol: undefined,
  xdai: '0x522de7D16aF21A255D2bC6EB24e6F9d93bD307a1',
}
