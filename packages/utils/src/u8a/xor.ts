/**
 * Apply an XOR on a list of arrays.
 *
 * @param inPlace if `true` overwrite first Array with result
 * @param list arrays to XOR
 */
export function u8aXOR(inPlace: boolean = false, ...list: Uint8Array[]): Uint8Array {
  if (!list.slice(1).every((array) => array.length == list[0].length)) {
    throw Error(`Uint8Arrays must not have different sizes`)
  }

  const result = inPlace ? list[0] : new Uint8Array(list[0].length)

  if (list.length == 2) {
    let index = 0

    let list0 = new DataView(list[0].buffer)
    let resultView = inPlace ? list0 : new DataView(result.buffer)

    let list1 = new DataView(list[1].buffer)

    for (; index + 8 <= list[0].length; index += 8) {
      resultView.setBigUint64(index, list0.getBigUint64(index) ^ list1.getBigUint64(index))
    }

    for (; index + 4 <= list[0].length; index += 4) {
      resultView.setUint32(index, list0.getUint32(index) ^ list1.getUint32(index))
    }

    for (; index + 2 <= list[0].length; index += 2) {
      resultView.setUint16(index, list0.getUint16(index) ^ list1.getUint16(index))
    }

    if (index < list[0].length) {
      resultView.setUint8(index, list0.getUint8(index) ^ list1.getUint8(index))
    }
  } else {
    let index = 0

    let listView = list.map((u8a) => new DataView(u8a.buffer))
    let resultView = inPlace ? listView[0] : new DataView(result.buffer)

    for (; index + 8 <= list[0].length; index += 8) {
      resultView.setBigUint64(index, listView[0].getBigUint64(index) ^ listView[1].getBigUint64(index))
      for (let j = 2; j < list.length; j++) {
        resultView.setBigUint64(index, resultView.getBigUint64(index) ^ listView[j].getBigUint64(index))
      }
    }

    for (; index + 4 <= list[0].length; index += 4) {
      resultView.setUint32(index, listView[0].getUint32(index) ^ listView[1].getUint32(index))
      for (let j = 2; j < list.length; j++) {
        resultView.setUint32(index, resultView.getUint32(index) ^ listView[j].getUint32(index))
      }
    }

    for (; index + 2 <= list[0].length; index += 2) {
      resultView.setUint16(index, listView[0].getUint16(index) ^ listView[1].getUint16(index))
      for (let j = 2; j < list.length; j++) {
        resultView.setUint16(index, resultView.getUint16(index) ^ listView[j].getUint16(index))
      }
    }

    if (index < list[0].length) {
      resultView.setUint8(index, listView[0].getUint8(index) ^ listView[1].getUint8(index))

      for (let j = 2; j < list.length; j++) {
        resultView.setUint8(index, resultView.getUint8(index) ^ listView[j].getUint8(index))
      }
    }
  }

  return result
}
