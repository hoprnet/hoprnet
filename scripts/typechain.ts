import path from 'path'
import { tsGenerator } from 'ts-generator'
import { TypeChain } from 'typechain/dist/TypeChain'

const root = path.join(__dirname, '..')

async function main() {
  const cwd = process.cwd()

  await tsGenerator(
    { cwd },
    new TypeChain({
      cwd,
      rawConfig: {
        files: path.join(root, 'node_modules/@hoprnet/hopr-ethereum/build/extracted/abis/*.json'),
        outDir: path.join(root, 'src/tsc/web3'),
        target: 'web3-v1'
      }
    })
  )
}

main().catch(console.error)
