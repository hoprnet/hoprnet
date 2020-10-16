export const HOPR_NETWORK = process.env['NEXT_PUBLIC_HOPR_NETWORK'] || 'basodino'
export const HOPR_DATABASE_URL = process.env['NEXT_PUBLIC_HOPR_DATABASE_URL'] || 'hopr-ch-develop'

export interface EnvironmentProps {
  COVERBOT_XDAI_THRESHOLD: number
  COVERBOT_VERIFICATION_CYCLE_IN_MS: number
  COVERBOT_DEBUG_MODE: boolean
  COVERBOT_CHAIN_PROVIDER: string
  COVERBOT_DEBUG_HOPR_ADDRESS: string
}

console.log('env', {
  HOPR_NETWORK,
  HOPR_DATABASE_URL
})
