// Copied from `core` to prevent from ESM import issues
// ESM support requires code changes within `hardhat-core`
export type NetworkOptions = {
  id: string
  description: string
  chain_id: number // >= 0
  live: boolean
  default_provider: string // a valid HTTP url pointing at a RPC endpoint
  etherscan_api_url?: string // a valid HTTP url pointing at a RPC endpoint
  max_fee_per_gas: string // The absolute maximum you are willing to pay per unit of gas to get your transaction included in a block, e.g. '10 gwei'
  max_priority_fee_per_gas: string // Tips paid directly to miners, e.g. '2 gwei'
  native_token_name: string
  hopr_token_name: string
  tags: string[]
}
