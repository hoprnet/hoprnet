import assert from 'assert'
import randomSubset from './randomSubset'
describe('testing random subset', function() {
    it('should return a subset with a filter function', function() {
        assert.deepEqual(
            randomSubset([1], 1),
            [1]
        )

        assert.deepEqual(
            randomSubset([1,2,3], 3).sort(),
            [1,2,3]
        )

        let array = []

        for (let i = 0; i < 30; i++) {
            array.push(i)
        }

        let result = randomSubset(array, 10, (value: number) => value % 2 == 0)

        assert(result.length == 10)

        assert(result.every(value => value % 2 == 0))

        let set = new Set<number>()
        array.forEach(value => {
            assert(0 <= value && value < 30)
            assert(!set.has(value))
            set.add(value)
        })
    })

    it('should return a subset', function() {
        assert.deepEqual(
            randomSubset([1, 2], 1, (value: number) => value == 1),
            [1]
        )

        assert.deepEqual(
            randomSubset([1,2,3], 3, (value: number) => [1,2,3].includes(value)).sort(),
            [1,2,3]
        )

        let array = []

        for (let i = 0; i < 30; i++) {
            array.push(i)
        }

        let result = randomSubset(array, 10)

        assert(result.length == 10)

        let set = new Set<number>()
        array.forEach(value => {
            assert(0 <= value && value < 30)
            assert(!set.has(value))
            set.add(value)
        })
    })
})
