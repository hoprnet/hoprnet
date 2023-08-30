/**
 *
 * @param input a string containing templated references to environment variables e.g. 'foo ${bar}'
 * @param vars a key-value vars storage object, e.g. { 'bar': 'bar_value' }
 * @returns a string with variables resolved to the actual values
 */
export function expandVars(input: string, vars: { [key: string]: any }) {
  return input.replace(/\$\{(.*)\}/g, (_, varName) => {
    if (!(varName in vars)) {
      throw new Error(`failed to expand vars in string '${input}', var ${varName} not defined`)
    }
    return vars[varName]
  })
}
