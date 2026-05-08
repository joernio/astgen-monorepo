import {decodePos, encodePos} from "../src/TscUtils"
import * as Defaults from "../src/Defaults"

describe("encodePos / decodePos", () => {
    it("round-trips representative positions", () => {
        const cases: [number, number][] = [
            [0, 0],
            [0, 1],
            [1, 0],
            [12, 27],
            [1234, 5678],
            [Defaults.MAX_FILE_SIZE_BYTES - 1, Defaults.MAX_FILE_SIZE_BYTES - 1],
        ]
        for (const [s, e] of cases) {
            const key = encodePos(s, e)
            expect(decodePos(key)).toEqual([s, e])
        }
    })

    it("round-trips a randomized sample within the supported range", () => {
        // POS_SHIFT (2^26) is more than 12x MAX_FILE_SIZE_BYTES (5MB), so any
        // position within a permitted file fits comfortably below the shift.
        const max = Defaults.MAX_FILE_SIZE_BYTES
        for (let i = 0; i < 200; i++) {
            const s = Math.floor(Math.random() * max)
            const e = Math.floor(Math.random() * max)
            const key = encodePos(s, e)
            expect(decodePos(key)).toEqual([s, e])
        }
    })

    it("produces distinct keys for distinct (start, end) pairs near the boundary", () => {
        const max = Defaults.MAX_FILE_SIZE_BYTES - 1
        expect(encodePos(max, 0)).not.toEqual(encodePos(0, max))
        expect(encodePos(max, max)).not.toEqual(encodePos(max - 1, max))
    })
})
