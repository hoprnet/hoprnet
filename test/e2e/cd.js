const util = require('util')
const execFile = util.promisify(require('child_process').execFile)
const setupEnv = require('./set-up-env');
const getAddressEnv = require('./get-address-env');

const DIR_NAME = './scripts/cd/'

const gcloudVmSetupScript = DIR_NAME + 'gcloud_vm_setup.sh'

const main = async () => {
  try {
    const envFromScripts = await setupEnv();
    const env = Object.assign({}, process.env, envFromScripts);
    await execFile(gcloudVmSetupScript, [], { env })
    const addressEnvFromScripts = await getAddressEnv();
    console.log(addressEnvFromScripts)
  } catch (e) {
    console.error('Error', e);
  }
}

main()
