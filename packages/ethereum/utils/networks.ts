import Web3 from 'web3'

export type PrivateNetwork = 'localhost'
export type PublicNetwork = 'mainnet' | 'kovan' | 'xdai' | 'matic' | 'binance' | 'ropsten' | 'goerli'
export type Network = PublicNetwork | PrivateNetwork
export type RpcOptions = {
  chainId: number
  httpUrl: string
  wsUrl: string
  gasPrice?: number
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
    ropsten: {
      chainId: 3,
      httpUrl: `https://ropsten.infura.io/v3/${infura}`,
      wsUrl: `wss://ropsten.infura.io/ws/v3/${infura}`
    },
    goerli: {
      chainId: 5,
      httpUrl: `https://goerli.infura.io/v3/${infura}`,
      wsUrl: `wss://goerli.infura.io/ws/v3/${infura}`
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
    },
    binance: {
      chainId: 56,
      httpUrl: 'https://bsc-dataseed.binance.org',
      wsUrl: 'wss://bsc-ws-node.nariox.org:443',
      gasPrice: Number(Web3.utils.toWei('20', 'gwei'))
    }
  }
}
