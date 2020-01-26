import dotenv from 'dotenv'
import dotenvExpand from 'dotenv-expand'

const myEnv = dotenv.config()
dotenvExpand(myEnv)

// Set contract address according to chosen network
process.env['CONTRACT_ADDRESS'] = process.env[`CONTRACT_ADDRESS_${process.env['NETWORK'].toUpperCase()}`]

// Set web3 provider info according to chosen network
process.env['PROVIDER'] = process.env[`PROVIDER_${process.env['NETWORK'].toUpperCase()}`]

const toDelete = Object.keys(process.env).filter((key: string) => key.startsWith('CONTRACT_ADDRESS_') || key.startsWith('PROVIDER_') || key.startsWith('GAS_PRICE_'))

// Delete unused environment properties
toDelete.forEach((key: string) => {
    delete process.env[key]
})
