import { join } from 'path'
import { spawn } from 'child_process'

export const ROOT = join(__dirname, '..')

/**
 * Runs a command as if you were running bash at this project's root folder.
 * TODO: remove this once we reface `core-ethereum`
 * @param cmd command
 */
async function runner(cmd: string): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    try {
      const [first, ...rest] = cmd.split(' ')
      const child = spawn(first, rest, {
        cwd: ROOT
      })

      child.stdout.setEncoding('utf8')
      child.stderr.setEncoding('utf8')

      child.stdout.on('data', console.log)
      child.stderr.on('data', console.error)

      child.on('exit', (code) => {
        if (code === 0) {
          resolve()
        } else {
          reject()
        }
      })
      child.on('error', reject)
    } catch (err) {
      reject(err)
    }
  })
}

export default runner
