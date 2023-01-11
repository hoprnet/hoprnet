/**
 * Repeatedly apply a function after a timeout
 * @param fn function to apply after every timeout
 * @param newTimeout function that returns the new timeout
 */
export function retimer(fn: () => Promise<void> | void, newTimeout: () => number, awaitPromise?: boolean): () => void {
  let timeout: NodeJS.Timeout

  const again = async () => {
    if (awaitPromise == true) {
      await fn()
    } else {
      fn()
    }
    timeout = setTimeout(again, newTimeout()).unref()
  }
  timeout = setTimeout(again, newTimeout()).unref()

  return () => clearTimeout(timeout)
}
