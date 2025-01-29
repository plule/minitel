"use strict"
/**
 * @file stream.js
 * @author Frédéric BISSON <zigazou@free.fr>
 * @version 1.0
 */

/**
 * @namespace Minitel
 */
var Minitel = Minitel || {}

/**
 * A Minitel Stream is an helper class which transforms any kind of JavaScript
 * type (string, number, array etc.) to an array of integers.
 *
 * A Minitel Stream is a queue.
 */
Minitel.Stream = class {

    /**
     * Constructor
     */
    constructor() {
        this.reset()
    }

    /**
     * Reset the stream
     */
    reset() {
        /**
         * An array of integer codes
         * @member {number[]}
         */
        this.items = new Array(10000)
        this.length = 0
    }

    /**
     * Converts the stream to an array of bytes
     * @return {number[]}
     */
    toArray() {
        return this.items.slice(0, this.length)
    }

    /**
     * @param {} item Any value to insert in the queue.
     */
    push(item) {
        let toPush

        if(typeof item === "number") {
            // Number
            toPush = item
        } else if(typeof item === "string" && item.length === 1) {
            // String
            toPush = item.charCodeAt(0)
        } else if(item instanceof Minitel.Stream) {
            // Stream
            range(item.length).forEach(i => {
                this.items[this.length] = item.items[i]
                this.length++
            })
        } else if(item !== undefined
                  && typeof item[Symbol.iterator] === "function") {
            // Iterable object
            for(let value of item) {
                this.push(value)
            }
        }

        if(toPush !== undefined) {
            if(Minitel.specialChars[toPush]) {
                // Convert special characters to Minitel codes
                Minitel.specialChars[toPush].forEach(v => {
                    this.items[this.length] = v
                    this.length++
                })
            } else if(toPush > 0x7f) {
                // Minitel does not understand values above 0x7f
                return
            } else {
                this.items[this.length] = toPush
                this.length++
            }
        }
    }

    /**
     * The shift method
     * @return {number}
     */
    shift() {
        this.length--
        return this.items.shift()
    }

    /**
     * The pop method
     * @return {number}
     */
    pop() {
        this.length--
        return this.items.pop()
    }

    /**
     * Generates a trimmed version of the current stream by removing every
     * control codes. It won't properly work when used on anything else than a
     * row.
     * @return {Stream} The trimmed version of the current stream
     */
    trimRow() {
        let lastChar = -1
        for(let i = 0; i < this.length; i++) {
            if(this.items[i] >= 0x20) {
                lastChar = i
                continue
            }

            if(this.items[i] === 0x12) {
                i++
                lastChar = i
                continue
            }

            if(this.items[i] === 0x1b) {
                i++
            }

            if(this.items[i] === 0x1f) {
                i += 2
            }
        }

        const trimmed = new Minitel.Stream()
        trimmed.push(this.items.slice(0, lastChar + 1))

        return trimmed
    }

    /**
     * Generate repeat commands while checking for overflow.
     *
     * @param {string} char Character to repeat
     * @param {int} count How many times to repeat
     */
    generateRepeat(char, count) {
        if(count === 1) {
            return char
        } else {
            let repeats = []

            while(count > 0x3F) {
                repeats = repeats.concat([0x12, 0x7F])
                count -= 0x3F
            }

            return repeats.concat([0x12, 0x40 + count])
        }
    }

    /**
     * Generates an optimized version of the current stream. It won't properly
     * work when used on anything else than a row.
     *
     * @param {boolean} moveFirst If True, the row is considered to be preceded
     *                            by a locate command. If false, the row is part
     *                            of a bigger stream.
     * @return {Stream} An optimized version of the current stream
     */
    optimizeRow(moveFirst) {
        let esc = false
        let count = 0
        let char = 0x00

        const current = {
            bg: moveFirst ? 0x50 : undefined,
            fg: moveFirst ? 0x47 : undefined,
            separated: moveFirst ? 0x59 : undefined,
            invert: moveFirst ? 0x5c : undefined,
            blink: moveFirst ? 0x49 : undefined,
            size: moveFirst ? 0x4c : undefined,
            charset: moveFirst ? 0x0f : undefined
        }

        const next = {
            bg: moveFirst ? 0x50 : undefined,
            fg: moveFirst ? 0x47 : undefined,
            separated: moveFirst ? 0x59 : undefined,
            invert: moveFirst ? 0x5c : undefined,
            blink: moveFirst ? 0x49 : undefined,
            size: moveFirst ? 0x4c : undefined,
            charset: moveFirst ? 0x0f : undefined
        }

        const optimized = new Minitel.Stream()

        range(this.length).forEach(i => {
            const item = this.items[i]

            // Ignores NUL characters.
            if(item === 0x00) return

            if(item === 0x1b) {
                // Found an Escape code sequence.
                esc = true
                return
            }

            if(esc) {
                // A code sequence has been started, read the action.
                if(item >= 0x40 && item <= 0x47) {
                    // Change foreground color.
                    next.fg = item
                } else if(item >= 0x50 && item <= 0x57) {
                    // Change background color.
                    next.bg = item
                } else if(item === 0x59 || item === 0x5a) {
                    // Enable/disable separated mosaic.
                    next.separated = item
                } else if(item === 0x5d || item === 0x5c) {
                    // Enable/disable video inverse.
                    next.invert = item
                } else if(item === 0x49 || item === 0x48) {
                    // Enable/disable blinking.
                    next.blink = item
                } else if(item >= 0x4c && item <= 0x4f) {
                    // Change character size.
                    next.size = item
                }

                // The Escape code sequence ends here.
                esc = false
            } else if(item < 0x20) {
                // Found a control code.
                if(item === 0x0e || item === 0x0f) {
                    // Select G0 or G1 character set.
                    next.charset = item
                    if(current.charset !== item) {
                        current.size = 0x4c
                        current.separated = 0x59
                    }
                } else {
                    // Flush any repeated character.
                    if(count > 0) {
                        optimized.push(this.generateRepeat(char, count))
                        count = 0
                    }

                    optimized.push(item)
                }
            } else {
                // Found a visible character.
                let attributeChange = false

                // Look for an attribute change.
                for(let attr in next) {
                    if(!next.hasOwnProperty(attr)) continue
                    if(next[attr] === undefined
                       || next[attr] === current[attr]) {
                        continue
                    }
                    attributeChange = true
                }

                // Flush any repeated character.
                if(count > 0 && (attributeChange || char !== item)) {
                    optimized.push(this.generateRepeat(char, count))
                    count = 0
                }

                // Watch every attribute.
                [
                    "charset",
                    "size",
                    "invert",
                    "bg",
                    "fg",
                    "separated",
                    "blink"
                ].forEach(
                    attr => {
                        if(next[attr] === undefined) return
                        if(current[attr] === next[attr]) return

                        // Ignore changing foreground color if a space character
                        // is output.
                        if(attr === "fg" && item === 0x20) {
                            if(current.charset === 0x0e) return
                            if(current.invert === 0x5c) return
                        }

                        if(attr !== "charset") optimized.push(0x1b)
                        optimized.push(next[attr])
                        current[attr] = next[attr]
                        next[attr] = undefined
                    }
                )

                if(char !== item) {
                    // Character has changed.
                    optimized.push(item)
                    char = item
                } else {
                    // Character is the same.
                    count++
                }
            }
        })

        // Flush any remaining repeated character.
        if(count > 0) {
            optimized.push(this.generateRepeat(char, count))
        }

        return optimized
    }
}

