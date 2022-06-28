import debug from 'debug'

const wrappedDebug = (namespace: any) => {
  return (message: any, ...parameters: any[]) => {
    return debug(namespace)(message, ...parameters)
  }
}

export { wrappedDebug as debug }
