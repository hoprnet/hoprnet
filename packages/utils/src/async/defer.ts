export class Defer<T> {
  public promise: Promise<T>
  private _resolve: (value?: T | PromiseLike<T>) => void
  private _reject: (reason?: any) => void

  constructor() {
    this.promise = new Promise<T>((resolve, reject) => {
      this._resolve = resolve
      this._reject = reject
    })
  }

  public resolve(value?: T | PromiseLike<T>): void {
    this._resolve(value)
  }

  public reject(reason?: any): void {
    this._reject(reason)
  }
}
