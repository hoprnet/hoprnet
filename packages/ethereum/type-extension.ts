import 'hardhat-deploy'

// Extend Hardhat's runtime environment type to include
// environment property
declare module 'hardhat/types/runtime' {
  interface HardhatRuntimeEnvironment {
    environment: string
  }
}
