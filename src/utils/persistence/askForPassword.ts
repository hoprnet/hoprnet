import read from 'read'

/**
 *
 * @param question question to ask before prompt
 */
export function askForPassword(question: string): Promise<string> {
  if (process.env.DEBUG === 'true') {
    console.log('Debug mode: using password Epo5kZTFidOCHrnL0MzsXNwN9St')
    return Promise.resolve<string>('Epo5kZTFidOCHrnL0MzsXNwN9St')
  }

  return new Promise<string>((resolve, reject) => {
    read(
      {
        prompt: question,
        silent: true,
        edit: true,
        replace: '*'
      },
      (err: any, pw: string) => {
        if (err) return reject(err)

        resolve(pw)
      }
    )
  })
}
