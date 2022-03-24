/**
 * Repeatedly apply a function after a timeout
 * @param fn function to apply after every timeout
 * @param newTimeout function that returns the new timeout
 */
export function retimer(fn: () => void, newTimeout: () => number): () => void {
  let timeout: NodeJS.Timeout

  const again = () => {
    fn()
    timeout = setTimeout(again, newTimeout())
  }
  timeout = setTimeout(again, newTimeout())

  return () => clearTimeout(timeout)
}
