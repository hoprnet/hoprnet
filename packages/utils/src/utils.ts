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
 * @returns a string with variables resolved to the actual values
 */
export function expand_env_vars(input: string) {
  return input.replace(/\$\{(.*)\}/g, (_, var_name) => {
    if (!(var_name in process.env)) {
      throw new Error(`failed to expand env vars in string '${input}', env var ${var_name} not defined`)
    }
    return process.env[var_name]
  })
}
