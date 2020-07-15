import type { CoreService } from './core.service'

/*
  A decorator function to check whether node is started,
  if not it will throw an error
*/
export function mustBeStarted(): MethodDecorator {
  return (
    _target: CoreService,
    _key: string,
    descriptor: TypedPropertyDescriptor<any>,
  ): TypedPropertyDescriptor<any> => {
    const originalFn = descriptor.value

    descriptor.value = function (...args: any[]) {
      if (!this.started) {
        throw Error('HOPR node is not started')
      }

      return originalFn.bind(this)(...args)
    }

    return descriptor
  }
}
