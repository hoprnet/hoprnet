export type PrivateNetwork = 'hardhat'
export type PublicNetwork = 'mainnet' | 'kovan' | 'xdai' | 'matic'
export type Network = PublicNetwork | PrivateNetwork
export type MigrationOptions = {
  shouldVerify: boolean
  mintUsing: 'minter' | 'faucet'
  revokeRoles: boolean
}
export type RpcOptions = {
  chainId: number
  httpUrl: string
  wsUrl: string
}

export const migrationOptions: { [key in Network]: MigrationOptions } = {
  hardhat: {
    shouldVerify: false,
    mintUsing: 'minter',
    revokeRoles: false
  },
  mainnet: {
    shouldVerify: true,
    mintUsing: 'faucet',
    revokeRoles: true
  },
  kovan: {
    shouldVerify: true,
    mintUsing: 'minter',
    revokeRoles: false
  },
  xdai: {
    shouldVerify: false,
    mintUsing: 'minter',
    revokeRoles: false
  },
  matic: {
    shouldVerify: false,
    mintUsing: 'minter',
    revokeRoles: false
  }
}

export function getRpcOptions(ops?: { infura?: string; maticvigil?: string }): { [key in PublicNetwork]: RpcOptions } {
  const infura = ops?.infura
  const maticvigil = ops?.maticvigil

  return {
    mainnet: {
      chainId: 1,
      httpUrl: `https://mainnet.infura.io/v3/${infura}`,
      wsUrl: `wss://mainnet.infura.io/v3/${infura}`
    },
    kovan: {
      chainId: 42,
      httpUrl: `https://kovan.infura.io/v3/${infura}`,
      wsUrl: `wss://kovan.infura.io/v3/${infura}`
    },
    xdai: {
      chainId: 100,
      httpUrl: `https://xdai.poanetwork.dev`,
      wsUrl: 'wss://xdai.poanetwork.dev/wss'
    },
    matic: {
      chainId: 137,
      httpUrl: `https://rpc-mainnet.maticvigil.com/v1/${maticvigil}`,
      wsUrl: `wss://rpc-mainnet.maticvigil.com/v1/${maticvigil}`
    }
  }
}
