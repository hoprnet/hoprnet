'use strict'

const { execFile } = require('child_process');


const waterfall = require('async/waterfall')
const each = require('async/each')
const some = require('async/some')
const parallel = require('async/parallel')

const fs = require('fs')

const files = ['PaymentChannel.sol']

module.exports = (cb) => {
    let contract;

    waterfall([
        (cb) => each(files, (file, cb) => rebuildIfNecessary(file, cb), cb),
        (cb) => parallel({
            abi: (cb) => fs.readFile(deriveBuildString(__dirname + '/' + files[0], 'abi'), cb),
            binary: (cb) => fs.readFile(deriveBuildString(__dirname + '/' + files[0], 'bin'), cb)
        }, cb)
    ], cb)
}


function rebuildIfNecessary(file, cb) {
    if (!fs.existsSync(__dirname + '/' + file))
        throw Error('File does not exists.')

    waterfall([
        (cb) => fs.stat(__dirname + '/' + file, cb),
        (sourceFileStats, cb) => {
            if (
                !fs.existsSync(__dirname + '/' + deriveBuildString(file, 'bin')) ||
                !fs.existsSync(__dirname + '/' + deriveBuildString(file, 'abi'))
            ) {
                compile(file, cb)
            } else {
                some(['abi', 'bin'], (suffix, cb) => fs.stat(__dirname + '/' + deriveBuildString(file, suffix), (err, stats) => {
                    if (err) {
                        cb(err)
                    } else {
                        cb(err, sourceFileStats.mtimeMs > stats.mtimeMs)
                    }
                }), (err, result) => {
                    if (err) { throw err }
                    if (result) {
                        compile(file, cb)
                    } else {
                        cb()
                    }
                })
            }
        }
    ], (err, stdout) => {
        if (err) { throw err }
        cb()
    })
}

function compile(file, cb) {
    execFile('solcjs', [file, '--abi', '--bin'], {
        cwd: __dirname
    }, cb)
}

function deriveBuildString(str, suffix) {
    const lastIndex = str.lastIndexOf('.sol')

    if (lastIndex < 0)
        throw Error('Please provide a file whose filename ends on \".sol\".')

    return str.slice(0, lastIndex).concat('_sol_Hopper.').concat(suffix)
}