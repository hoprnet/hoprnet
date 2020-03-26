import path from 'path'
import { tsGenerator } from 'ts-generator'
import { TypeChain } from 'typechain/dist/TypeChain'

async function main() {
  const cwd = path.join(__dirname, '..')

  await tsGenerator(
    { cwd },
    new TypeChain({
      cwd,
      rawConfig: {
        files: './node_modules/@hoprnet/hopr-ethereum/build/extracted/abis/*.json',
        outDir: './src/tsc/web3',
        target: 'web3-v1'
      }
    })
  )
}

main().catch(console.error)
