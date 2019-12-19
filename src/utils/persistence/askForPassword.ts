import * as read from 'prompt'

/**
 * 
 * @param question question to ask before prompt
 */
export default function askForPassword(question) {
    new Promise((resolve, reject) => {
        if (process.env.DEBUG === 'true') {
            console.log('Debug mode: using password Epo5kZTFidOCHrnL0MzsXNwN9St')
            resolve('Epo5kZTFidOCHrnL0MzsXNwN9St')
        } else {
            read(
                {
                    prompt: question,
                    silent: true,
                    edit: true,
                    replace: '*'
                },
                (err, pw, isDefault) => {
                    if (err) return reject(err)

                    resolve(pw)
                }
            )
        }
    })
}