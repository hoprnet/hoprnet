import path from 'path'
import { tsGenerator } from 'ts-generator'
import { TypeChain } from 'typechain/dist/TypeChain'

async function main() {
  const cwd = path.join(__dirname, '..')
  console.log(cwd)

  await tsGenerator(
    { cwd },
    new TypeChain({
      cwd,
      rawConfig: {
        files: path.join(cwd, 'node_modules/@hoprnet/hopr-ethereum/build/extracted/abis/*.json'),
        outDir: './src/tsc/web3',
        target: 'web3-v1'
      }
    })
  )
}

main().catch(console.error)
