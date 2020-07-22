import dotenv from 'dotenv'
import dotenvParse from 'dotenv-parse-variables'

const result = dotenv.config()
if (result.error) {
  throw result.error
}

dotenvParse(result.parsed) as Record<string, any>

export const API_URL: string = process.env.API_URL ?? '127.0.0.1:50051'
