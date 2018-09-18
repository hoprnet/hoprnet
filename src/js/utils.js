'use strict'

const last = require('lodash.last')


module.exports.parseJSON = function (str) {
    return JSON.parse(str, (key, value) => {
        if (value && value.type === 'Buffer') {
            return Buffer.from(value.data)
        }
        
        return value
    })
}

module.exports.bufferXOR = function(buf1, buf2) {
    if (!Buffer.isBuffer(buf1) || !Buffer.isBuffer(buf2))
        throw Error('Input values have to be provided as Buffers. Got ' + typeof buf1 + ' and ' + typeof buf2)
    
    if (buf1.length !== buf2.length)
        throw Error('Buffer must have the same length. Got buffers of length ' + buf1.length + ' and ' + buf2.length)

    return buf1.map((elem, index) => (elem ^ buf2[index]))
}

module.exports.bufferXOR_in_place = function(result, buf2) {
    if (!Buffer.isBuffer(result) || !Buffer.isBuffer(buf2))
        throw Error('Input values have to be provided as Buffers. Got ' + typeof result + ' and ' + typeof buf2)
    
    if (result.length !== buf2.length)
        throw Error('Buffer must have the same lenght. Got buffers of length ' + result.length + ' and ' + buf2.length)

    result.forEach((elem , index, elems) => {
        elems[index] = elem ^ buf2[index]
    })

    return result
}

module.exports.bufferADD_in_place = function (buf, add) {
    if (!Buffer.isBuffer(buf))
        throw Error('Expected Buffer as input. Got ' + typeof buf)
    
    //  shortcut
    if (last(buf) + add <= 255) {
        buf[buf.length - 1] = last(buf) + add

        return buf
    }
    
    throw Error('TODO')
    // let maxlength = Math.max(buf.length, add.length)

    // if (maxlength > buf.length) {
    //     buf = Buffer.concat([Buffer.allocate(maxlength - buf.length).fill(0), buf], maxlength)
    // }

    // let overflow = 0
    // for (let i = 0; i < add.length; i++) {
    //     if (buf[i] + add[i] + overflow)
    //     buf[i] = buf[i] 
    //     overflow = buf[i] + add[i] - 255

    // }
    // if (buf.l)
    // let overflow = add - last(buf) 

    // while(overflow > 255) {

    // }

}

