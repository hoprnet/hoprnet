import chalk from 'chalk'
import { cli_options } from './cliOptions'

const FIRST_OPTION_OFFSET = 1
const SECOND_OPTION_OFFSET = 5
const EXTRA_PADDING = 2

export function displayHelp() {
  let firstOptionMaxLength = 0
  let secondOptionMaxLength = 0

  for (let i = 0; i < cli_options.length; i++) {
    if (cli_options[i][0] != null && cli_options[i][0].length > firstOptionMaxLength) {
      firstOptionMaxLength = cli_options[i][0].length
    }

    if (cli_options[i][2] != null) {
      if (cli_options[i][1] != null && cli_options[i][1].length + cli_options[i][2].length > secondOptionMaxLength) {
        secondOptionMaxLength = cli_options[i][1].length + cli_options[i][2].length
      }
    } else {
      if (cli_options[i][1] != null && cli_options[i][1].length > secondOptionMaxLength) {
        secondOptionMaxLength = cli_options[i][1].length
      }
    }
  }

  let str = ''

  const offset = firstOptionMaxLength + FIRST_OPTION_OFFSET + secondOptionMaxLength + SECOND_OPTION_OFFSET

  for (let i = 0; i < cli_options.length; i++) {
    str += (cli_options[i][0] || '').padEnd(firstOptionMaxLength + FIRST_OPTION_OFFSET, ' ')
    str += (cli_options[i][1] != null
      ? '[' + cli_options[i][1] + ']' + (cli_options[i][2] != null ? ' ' + cli_options[i][2] : '')
      : ''
    ).padEnd(secondOptionMaxLength + SECOND_OPTION_OFFSET, ' ')

    if (offset + cli_options[i][3].length > process.stdout.columns) {
      const words = cli_options[i][3].split(/\s+/)

      const allowance = process.stdout.columns - offset
      let length = 0
      for (let j = 0; j < words.length; j++) {
        if (words[j].length > allowance) {
          str += words[j] + '\n'
          continue
        }

        if (length + words[j].length < allowance) {
          str += words[j]
          length += words[j].length
        } else {
          str += '\n' + ''.padEnd(offset, ' ') + words[j]
          length = words[j].length
        }

        if (j < words.length - 1) {
          str += ' '
          length++
        }
      }
    } else {
      str += cli_options[i][3]
    }

    if (i < cli_options.length - 1) {
      str += '\n'
    }
  }

  console.log(str)
}

export function getPaddingLength(items: string[]): number {
  return Math.max(...items.map((str) => str.length)) + EXTRA_PADDING
}

export function styleValue(value: any): string {
  switch (true) {
    case typeof value === 'boolean':
      return chalk.hex('#BA55D3')(value)
    case typeof value === 'number':
      return chalk.blue(value)
    case typeof value === 'string':
      return chalk.yellow(value)
    default:
      return String(value)
  }
}

export function getOptions(
  options: { value: any; description?: string }[],
  style: 'compact' | 'vertical' = 'compact'
): string[] {
  if (style === 'compact') {
    return [`Options: ${options.map((o) => String(o.value)).join('|')}`]
  } else {
    const padding = getPaddingLength(options.map((o) => String(o.value)))

    return [
      'Options:',
      ...options.map((option, index) => {
        return [
          // needed to preperly format the array
          '\n',
          '- ',
          styleValue(String(option.value).padEnd(padding)),
          option.description,
        ].join('')
      }),
    ]
  }
}
