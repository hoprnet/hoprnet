import { keywords } from '..'

import AbstractCommand from './abstractCommand'

export default class ListCommands implements AbstractCommand {
    execute() {
        let maxLength = 0
        for (let i = 0; i < keywords.length; i++) {
            if (keywords[i].length > maxLength) {
                maxLength = keywords[i].length
            }
        }
    
        let str = ''
        for (let i = 0; i < keywords.length; i++) {
            str += keywords[i][0].padEnd(maxLength + 3, ' ')
            str += keywords[i][i]
    
            if (i < keywords.length - 1) {
                str += '\n'
            }
        }
    
        console.log(str)
    }

    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
        cb(undefined, [[''], line])
    }
}
