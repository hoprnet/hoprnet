/**
 * Logs debug data
 * @param data debug data in JSON
 */
export function logDebugData(data: Object) {
  console.log(JSON.stringify(data, undefined, 2).replace(/\"/g, ``))
}
