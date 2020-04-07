import chalk from 'chalk'

/**
 * Starts an interval after a timeout.
 * 
 * @param msg message to display
 */
export function startDelayedInterval(msg: string): () => void {
    let interval: NodeJS.Timeout
    let timeout = setTimeout(() => {
      process.stdout.write(`${chalk.green(msg)}\n`)
      interval = setInterval(() => {
        process.stdout.write(chalk.green('.'))
      }, 1000)
    }, 2 * 1000)
  
    return () => {
      clearTimeout(timeout)
      clearInterval(interval)
    }
  }