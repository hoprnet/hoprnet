import { join } from 'path'
import readPkgUp from 'read-pkg-up'

const pkg = readPkgUp.sync({
  cwd: join(__dirname, '..'),
})

const corePkg = readPkgUp.sync({
  cwd: join(__dirname, '..', 'node_modules', '@hoprnet', 'hopr-core'),
})

const runtimePkg = readPkgUp.sync({
  cwd: join(__dirname, '.'),
})

const runtimeCorePkg = readPkgUp.sync({
  cwd: join(__dirname, '.', 'hopr-core'),
})

const dependencies = ((pkg, corePkg) => ({
  // chat
  '@hoprnet/hopr-chat': pkg.packageJson.version,
  '@hoprnet/hopr-core': pkg.packageJson.dependencies['@hoprnet/hopr-core'],
  '@hoprnet/hopr-utils': pkg.packageJson.dependencies['@hoprnet/hopr-utils'],
  '@hoprnet/hopr-core-connector-interface': pkg.packageJson.dependencies['@hoprnet/hopr-core-connector-interface'],
  // core
  '@hoprnet/hopr-core-ethereum': corePkg.packageJson.dependencies['@hoprnet/hopr-core-ethereum'],
}))(pkg || runtimePkg, corePkg || runtimeCorePkg)

export default dependencies;
