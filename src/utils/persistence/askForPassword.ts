import read from 'read'

/**
 *
 * @param question question to ask before prompt
 */
export function askForPassword(question: string): Promise<string> {
  if (process.env.DEVEVLOP_MODE === 'true') {
    console.log('Debug mode: using password Epo5kZTFidOCHrnL0MzsXNwN9St')
    return Promise.resolve('Epo5kZTFidOCHrnL0MzsXNwN9St')
  }

  return new Promise<string>((resolve, reject) => {
    read(
      {
        prompt: question + ' (Password will not be echoed.)\n  password:',
        silent: true,
        edit: true
      },
      (err: any, pw: string) => {
        if (err) {
          return reject(err)
        }

        resolve(pw)
      }
    )
  })
}
