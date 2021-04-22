import read from 'read'

/**
 *
 * @param question question to ask before prompt
 */
export function askForPassword(question: string): Promise<string> {
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
