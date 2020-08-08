import dotenv from 'dotenv'
import dotenvParse from 'dotenv-parse-variables'

let parsed: {
  API_URL: string
  BOT_NAME: string
} = {
  API_URL: '127.0.0.1:50051',
  BOT_NAME: 'randobot'
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
