import { performance } from 'perf_hooks'
import Hjson from 'hjson'
import fs from 'fs'

export function timer(fn: () => void): number {
  const start = performance.now()
  fn()
  const end = performance.now() - start
  return end
}

/**
 *
 * @param input a string containing templated references to environment variables e.g. 'foo ${bar}'
 * @param vars a key-value vars storage object, e.g. { 'bar': 'bar_value' }
 * @returns a string with variables resolved to the actual values
 */
export function expandVars(input: string, vars: { [key: string]: any }) {
  return input.replace(/\$\{(.*)\}/g, (_, varName) => {
    if (!(varName in vars)) {
      throw new Error(`failed to expand vars in string '${input}', var ${varName} not defined`)
    }
    return vars[varName]
  })
}

/**
 * loads JSON data from file
 * @param file_path json file to load
 * @returns object parsed from JSON data
 * @throws if unable to open the file the JSON data is malformed
 */
export function loadJson(file_path: string): any {
  const content = fs.readFileSync(file_path, 'utf-8')
  return Hjson.parse(content)
}