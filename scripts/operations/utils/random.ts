import path from 'path'
import { spawn } from 'child_process'

export const bash = (cmd: string): Promise<void> => {
  return new Promise<void>((resolve, reject) => {
    const [first, ...rest] = cmd.split(' ')
    const child = spawn(first, rest)

    child.stdout.setEncoding('utf8')
    child.stderr.setEncoding('utf8')

    child.stdout.on('data', console.log)
    child.stderr.on('data', console.error)

    child.on('close', resolve)
    child.on('exit', resolve)
    child.on('error', reject)
  })
}

export const root = path.join(__dirname, '..', '..', '..')
