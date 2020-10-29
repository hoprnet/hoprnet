import path from 'path'
import { spawn } from 'child_process'

export const root = path.join(__dirname, '..', '..', '..')

export const bash = (cmd: string): Promise<void> => {
  return new Promise<void>((resolve, reject) => {
    try {
      const [first, ...rest] = cmd.split(' ')
      const child = spawn(first, rest, {
        cwd: root,
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

export const getOperations = () => {
  return ['build', 'coverage', 'fund', 'migrate', 'network', 'test', 'verify']
}
