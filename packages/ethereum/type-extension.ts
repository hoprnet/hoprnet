import { EthereumProvider } from 'hardhat/types'
import { Address, DeploymentsExtension } from 'hardhat-deploy/types'

declare module 'hardhat/types/runtime' {
  interface HardhatRuntimeEnvironment {
    environment: string
    deployments: DeploymentsExtension
    getNamedAccounts: () => Promise<{
      [name: string]: Address
    }>
    getUnnamedAccounts: () => Promise<string[]>
    getChainId(): Promise<string>
    companionNetworks: {
      [name: string]: {
        deployments: DeploymentsExtension
        getNamedAccounts: () => Promise<{
          [name: string]: Address
        }>
        getUnnamedAccounts: () => Promise<string[]>
        getChainId(): Promise<string>
        provider: EthereumProvider
      }
    }
  }

  interface Network {
    live: boolean
    saveDeployments: boolean
    tags: Record<string, boolean>
    deploy: string[]
    companionNetworks: { [name: string]: string }
  }
}
