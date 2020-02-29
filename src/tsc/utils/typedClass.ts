/*
  https://github.com/microsoft/TypeScript/issues/34516
  https://github.com/microsoft/TypeScript/issues/33892
*/

function typedClass<T>() {
  return <U extends T>(constructor: U) => {
    constructor
  }
}

export default typedClass
