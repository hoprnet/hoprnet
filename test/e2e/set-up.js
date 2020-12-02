const execFile = require('child_process').execFile
const setupEnv = require('./set-up-env');

const DIR_NAME = './scripts/cd/'

const gcloudEnvScript = DIR_NAME + 'gcloud_vm.sh'

const main = async () => {
  try {
    const env = await setupEnv();
    console.log(env);
    const child = execFile(gcloudEnvScript, [], { env })
    child.stdout.on('data', function (data) {
      console.log(data.toString());
    });
    child.stderr.on('data', function (data) {
      console.log(data.toString());
    });
  } catch (e) {
    console.error('Error', e);
  }
}

main();