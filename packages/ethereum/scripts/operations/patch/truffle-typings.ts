/*
  patch types: https://github.com/ethereum-ts/truffle-typings/pull/13#issuecomment-550325019
*/
import { promisify } from 'util'
import { join } from 'path'
import { readFile, writeFile } from 'fs'
import { root } from '../utils'

const typesFile = join(root, 'node_modules', 'truffle-typings', 'index.d.ts')

export default async () => {
  const data = await promisify(readFile)(typesFile, 'utf8')

  const result = data.replace('import("web3");', 'import("web3").default;')

  await promisify(writeFile)(typesFile, result, 'utf8')
  console.log('Successfully patched truffle-typings library!')
}
