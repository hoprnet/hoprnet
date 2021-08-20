import { performance } from 'perf_hooks'

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
  return input.replace(/\$\{(.*)\}/g, (_, var_name) => {
    if (!(var_name in vars)) {
      throw new Error(`failed to expand vars in string '${input}', var ${var_name} not defined`)
    }
    return vars[var_name]
  })
}
