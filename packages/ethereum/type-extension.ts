import 'hardhat-deploy'
import type { BigNumber } from 'ethers'

// Extend Hardhat's runtime environment type to include
// environment property
declare module 'hardhat/types/runtime' {
  interface HardhatRuntimeEnvironment {
    maxFeePerGas: BigNumber
    maxPriorityFeePerGas: BigNumber
    environment: string
  }
}
