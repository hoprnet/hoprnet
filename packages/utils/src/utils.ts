import { performance } from 'perf_hooks'

export function timer(fn: () => void): number {
  const start = performance.now()
  fn()
  const end = performance.now() - start
  return end
}
