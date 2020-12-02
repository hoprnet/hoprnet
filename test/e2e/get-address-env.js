const util = require('util')
const execFile = util.promisify(require('child_process').execFile)
const setupEnv = require('./set-up-env')
const utils = require('./utils')

const DIR_NAME = './scripts/cd/'

const releaseEnvScript = DIR_NAME + 'release_env.sh'
const gcloudVmStopScript = DIR_NAME + 'gcloud_vm_stop.sh'

const main = async () => {
  try {
    const envMap = {}
    const envFromScripts = await setupEnv()
    const env = Object.assign({}, process.env, envFromScripts)
    await execFile(gcloudVmStopScript, [], { env })
    return envMap
  } catch (e) {
    console.error('Error', e)
  }
}

module.exports = main
