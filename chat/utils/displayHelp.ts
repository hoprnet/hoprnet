import { cli_options } from '..'

export function displayHelp() {
  let length = 0

  let currentLength
  for (let i = 0; i < cli_options.length; i++) {
    currentLength = cli_options[i][0].length + (cli_options[i][1] != null ? cli_options[i][1].length : 0)

    if (currentLength > length) {
        length = currentLength
    }
  }

  let str = ''
  for (let i = 0; i < cli_options.length; i++) {
    if (cli_options[i][1] != null) {
        str += (cli_options[i][0] + ' [' + cli_options[i][1] + ']').padEnd(length + 7, ' ')
    } else {
        str += cli_options[i][0].padEnd(length + 7, ' ')
    }

    str += cli_options[i][2]

    if (i < cli_options.length - 1) {
      str += '\n'
    }
  }

  console.log(str)
}
