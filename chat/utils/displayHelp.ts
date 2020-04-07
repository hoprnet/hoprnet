import { cli_options } from '..'

export function displayHelp() {
    let maxCommandLength = 0
    let maxAliasLength = 0

    for (let i = 0; i < cli_options.length; i++) {
        if (cli_options[i][0].length > maxCommandLength) {
            maxCommandLength = cli_options[i][0].length
        }

        if (cli_options[i][1].length > maxAliasLength) {
            maxAliasLength = cli_options[i][1].length
        }
    }

    let str = ''
    for (let i = 0; i < cli_options.length; i++) {
        str += cli_options[i][0].padEnd(maxCommandLength + 1, ' ')
        str += cli_options[i][1].padEnd(maxAliasLength + 1, ' ') 
        str += cli_options[i][2]

        if (i < cli_options.length - 1) {
            str += '\n'
        }
    }

    console.log(str)
}