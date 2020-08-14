import dotenv from 'dotenv'
import dotenvParse from 'dotenv-parse-variables'

let parsed: {
  API_URL: string
  BOT_NAME: string
  TWITTER_API_KEY: string
  TWITTER_API_SECRET: string
  TWITTER_API_ACCESS_TOKEN: string
  TWITTER_API_ACCESS_TOKEN_SECRET: string
  PRIVATE_KEY: string
  ACCOUNT_ADDRESS: string
  INFURA_PROJECT_ID: string
  CHAIN: string
  DAI_ADDRESS: string
  CAMPAIGN_ID: number
} = {
  API_URL: '127.0.0.1:50051',
  BOT_NAME: 'randobot',
  TWITTER_API_KEY: '',
  TWITTER_API_SECRET: '',
  TWITTER_API_ACCESS_TOKEN: '',
  TWITTER_API_ACCESS_TOKEN_SECRET: '',
  PRIVATE_KEY: '',
  ACCOUNT_ADDRESS: '',
  INFURA_PROJECT_ID: '',
  CHAIN: 'mainnet',
  DAI_ADDRESS: '6b175474e89094c44da98b954eedeac495271d0f',
  CAMPAIGN_ID: 1,
}

try {
  const result = dotenv.config()
  if (!result.error) {
    for (const k in result.parsed) {
      process.env[k] = result.parsed[k]
    }
  }
} catch {}

parsed = {
  ...parsed,
  ...(dotenvParse(process.env) as typeof parsed),
}

export const API_URL = parsed.API_URL
export const BOT_NAME = parsed.BOT_NAME
export const TWITTER_API_KEY = parsed.TWITTER_API_KEY
export const TWITTER_API_SECRET = parsed.TWITTER_API_SECRET
export const TWITTER_API_ACCESS_TOKEN = parsed.TWITTER_API_ACCESS_TOKEN
export const TWITTER_API_ACCESS_TOKEN_SECRET = parsed.TWITTER_API_ACCESS_TOKEN_SECRET
export const PRIVATE_KEY = parsed.PRIVATE_KEY
export const ACCOUNT_ADDRESS = parsed.ACCOUNT_ADDRESS
export const INFURA_PROJECT_ID = parsed.INFURA_PROJECT_ID
export const CHAIN = parsed.CHAIN
export const DAI_ADDRESS = parsed.DAI_ADDRESS
export const CAMPAIGN_ID = parsed.CAMPAIGN_ID
