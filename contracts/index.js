'use strict'

const { execFile } = require('child_process');
const { waterfall, each, some, parallel } = require('async')
const { readFile, existsSync, stat } = require('fs')

const sourceFiles = ['PaymentChannel.sol']

module.exports = (cb) => waterfall([
    (cb) => each(sourceFiles, (file, cb) => rebuildIfNecessary(file, cb), cb),
    (cb) => parallel({
        abi: (cb) => readFile(deriveBuildString(__dirname + '/' + sourceFiles[0], 'abi'), cb),
        binary: (cb) => readFile(deriveBuildString(__dirname + '/' + sourceFiles[0], 'bin'), cb)
    }, cb)
], cb)


function rebuildIfNecessary(file, cb) {
    if (!existsSync(__dirname + '/' + file))
        throw Error('File does not exists.')

    waterfall([
        (cb) => stat(__dirname + '/' + file, cb),
        (sourceFileStats, cb) => {
            if (
                !existsSync(__dirname + '/' + deriveBuildString(file, 'bin')) ||
                !existsSync(__dirname + '/' + deriveBuildString(file, 'abi'))
            ) {
                compile(file, cb)
            } else {
                some(['abi', 'bin'], (suffix, cb) => stat(__dirname + '/' + deriveBuildString(file, suffix), (err, stats) => {
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