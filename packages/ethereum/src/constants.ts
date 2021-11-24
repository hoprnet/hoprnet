export type PublicNetworks = 'xdai' | 'goerli'
export type Networks = 'hardhat' | PublicNetworks

/**
 * testing = for ganache / hardhat powered chains which do not auto mine
 * development = chains which automine - may or may not be public chains
 * staging = chain should be treated as production chain
 * production = our current production chain
 */
export type DeploymentTypes = 'testing' | 'development' | 'staging' | 'production'
export type NetworkTag = DeploymentTypes | 'etherscan'
