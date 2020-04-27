/*
  extract abi and bytecode into seperate files
*/
import { join } from 'path'
import { readdir, readFile, writeFile, mkdir } from 'fs'
import { promisify } from 'util'
import { root } from './utils'

type Item = {
  name: string
  value: any
}

const inputPath = join(root, 'build', 'contracts')
const outputPath = join(root, 'build', 'extracted')

const getData = async () => {
  return promisify(readdir)(inputPath)
    .then((list) => {
      return list
        .filter((file) => file.startsWith('Hopr'))
        .map((file) => {
          return join(inputPath, file)
        })
    })
    .then((files) => {
      return Promise.all(
        files.map(async (file) => {
          return promisify(readFile)(file, {
            encoding: 'utf8',
          })
            .then((txt) => JSON.parse(txt))
            .then((json) => {
              return {
                contractName: json.contractName,
                abi: json.abi,
                bytecode: json.bytecode,
              }
            })
        })
      )
    })
}

const writeData = async (folder: string, items: Item[]) => {
  await promisify(mkdir)(join(outputPath, folder), {
    recursive: true,
  })

  return Promise.all(
    items.map((item) => {
      return promisify(writeFile)(join(outputPath, folder, `${item.name}.json`), JSON.stringify(item.value, null, 2))
    })
  )
}

export default async () => {
  const data = await getData()

  const { abis, bytecodes } = data.reduce<{
    [key: string]: Item[]
  }>(
    (result, output) => {
      result.abis.push({
        name: output.contractName,
        value: output.abi,
      })

      result.bytecodes.push({
        name: output.contractName,
        value: output.bytecode,
      })

      return result
    },
    {
      abis: [],
      bytecodes: [],
    }
  )

  return Promise.all([writeData('abis', abis), writeData('bytecodes', bytecodes)])
}
