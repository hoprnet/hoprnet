import { performance } from 'perf_hooks'

export function timer(fn: () => void): number {
  const start = performance.now()
  fn()
  const end = performance.now() - start
  return end
}

export function expand_env_vars(input: string) {
  return input.replace(/\$\{(.*)\}/g, (_, var_name) => {
    return process.env[var_name]
  })
}
