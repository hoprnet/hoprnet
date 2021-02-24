/**
 * Apply an XOR on a list of arrays.
 *
 * @param inPlace if `true` overwrite first Array with result
 * @param list arrays to XOR
 */
export function u8aXOR(inPlace: boolean = false, ...list: Uint8Array[]): Uint8Array {
  const list0length = list[0].length

  if (!list.slice(1).every((array) => array.length == list0length)) {
    throw Error(`Uint8Arrays must not have different sizes`)
  }

  let index = 0

  let result: Uint8Array
  let resultView: DataView

  let listView = list.map((u8a) => new DataView(u8a.buffer, u8a.byteOffset))

  if (!inPlace) {
    result = new Uint8Array(list0length)
    resultView = new DataView(result.buffer, result.byteOffset)
  } else {
    result = list[0]
    resultView = listView[0]
  }

  const listLength = list.length

  for (; index + 8 <= list0length; index += 8) {
    resultView.setBigUint64(index, listView[0].getBigUint64(index) ^ listView[1].getBigUint64(index))
    for (let j = 2; j < listLength; j++) {
      resultView.setBigUint64(index, resultView.getBigUint64(index) ^ listView[j].getBigUint64(index))
    }
  }

  for (; index + 4 <= list0length; index += 4) {
    resultView.setUint32(index, listView[0].getUint32(index) ^ listView[1].getUint32(index))
    for (let j = 2; j < listLength; j++) {
      resultView.setUint32(index, resultView.getUint32(index) ^ listView[j].getUint32(index))
    }
  }

  for (; index + 2 <= list0length; index += 2) {
    resultView.setUint16(index, listView[0].getUint16(index) ^ listView[1].getUint16(index))
    for (let j = 2; j < listLength; j++) {
      resultView.setUint16(index, resultView.getUint16(index) ^ listView[j].getUint16(index))
    }
  }

  if (index < list0length) {
    resultView.setUint8(index, listView[0].getUint8(index) ^ listView[1].getUint8(index))

    for (let j = 2; j < listLength; j++) {
      resultView.setUint8(index, resultView.getUint8(index) ^ listView[j].getUint8(index))
    }
  }

  return result
}
