export const HOPR_NETWORK = process.env['NEXT_PUBLIC_HOPR_NETWORK'] || 'basodino-develop'
export const HOPR_DATABASE_URL = process.env['NEXT_PUBLIC_HOPR_DATABASE_URL'] || 'hopr-coverbot'

export interface EnvironmentProps {
  COVERBOT_XDAI_THRESHOLD: number
  COVERBOT_VERIFICATION_CYCLE_IN_MS: number
  COVERBOT_DEBUG_MODE: boolean
  COVERBOT_CHAIN_PROVIDER: string
  COVERBOT_DEBUG_HOPR_ADDRESS: string
  COVERBOT_TIMESTAMP: number
}

console.log('env', {
  HOPR_NETWORK,
  HOPR_DATABASE_URL,
})
