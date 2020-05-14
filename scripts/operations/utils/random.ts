import path from 'path'
import { spawn } from 'child_process'
import networks from '../../../truffle-networks.json'

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

      child.on('exit', resolve)
      child.on('error', reject)
    } catch (err) {
      reject(err)
    }
  })
}

export const getContractNames = () => {
  return ['HoprChannels', 'HoprMinter', 'HoprToken']
}

export const getOperations = () => {
  return ['patch', 'build', 'coverage', 'fund', 'migrate', 'network', 'test', 'verify']
}

export const isLocalNetwork = (network: string) => {
  return !!Object.entries(networks)
    .filter(([, config]) => config.network_id === '*')
    .find(([name]) => name === network)
}
