/*
  patch types: https://github.com/trufflesuite/ganache-core/issues/465
*/
import { promisify } from 'util'
import { join } from 'path'
import { readFile, writeFile } from 'fs'
import { root } from '../utils'

const typesFile = join(root, 'node_modules', 'ganache-core', 'typings', 'index.d.ts')

export default async () => {
  const data = await promisify(readFile)(typesFile, 'utf8')

  const result = data.replace(
    'import { Provider as Web3Provider } from "web3/providers";',
    'import { WebsocketProvider as Web3Provider } from "web3-core";'
  )

  await promisify(writeFile)(typesFile, result, 'utf8')
  console.log('Successfully patched ganache-core library!')
}
