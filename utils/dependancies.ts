import { join } from 'path'
import readPkgUp from 'read-pkg-up'

const pkg = readPkgUp.sync({
  cwd: join(__dirname, '..'),
})

const corePkg = readPkgUp.sync({
  cwd: join(__dirname, '..', 'node_modules', '@hoprnet', 'hopr-core'),
})

console.log(pkg.packageJson)
console.log(corePkg.packageJson)

export default {
  // chat
  '@hoprnet/hopr-chat': pkg.packageJson.version,
  '@hoprnet/hopr-core': pkg.packageJson.dependencies['@hoprnet/hopr-core'],
  '@hoprnet/hopr-utils': pkg.packageJson.dependencies['@hoprnet/hopr-utils'],
  '@hoprnet/hopr-core-connector-interface': pkg.packageJson.dependencies['@hoprnet/hopr-core-connector-interface'],
  // core
  '@hoprnet/hopr-core-ethereum': corePkg.packageJson.dependencies['@hoprnet/hopr-core-ethereum'],
}
