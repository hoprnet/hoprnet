const util = require('util')
const execFile = util.promisify(require('child_process').execFile)

const RELEASE_VERSION = process.env.RELEASE_VERSION || require(process.cwd() + '/packages/hoprd/package.json').version
const BS_PASSWORD = process.env.BS_PASSWORD || 'switzerland'
const DIR_NAME = './scripts/cd/'

const releaseEnvScript = DIR_NAME + 'release_env.sh'
const gcloudEnvScript = DIR_NAME + 'gcloud_env.sh'

const parseEnvs = (stdout) => stdout.split('\n').map((line) => line.length > 0 && line.split('='))

const parseStdout = (stdout, envMap) =>
  parseEnvs(stdout)
    .filter(Boolean)
    .map((envs) => (([env, value]) => (envMap[env] = value))(envs))

const parsePromises = (envMap) => (envs) => envs.forEach(({ stdout, stderr }) => parseStdout(stdout, envMap))

const main = async () => {
  try {
    const envMap = {}
    envMap.RELEASE_VERSION = RELEASE_VERSION
    envMap.BS_PASSWORD = BS_PASSWORD
    await Promise.all([execFile(releaseEnvScript), execFile(gcloudEnvScript)]).then(parsePromises(envMap))
    return envMap
  } catch (e) {
    console.error('Error', e)
    return {}
  }
}

module.exports = main
