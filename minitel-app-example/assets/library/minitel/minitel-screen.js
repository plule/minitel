"use strict"
/**
 * @file MinitelScreen
 * @author Frédéric BISSON <zigazou@free.fr>
 * @version 1.0
 */

/**
 * @namespace Minitel
 */
var Minitel = Minitel || {}

/**
 * @class MinitelScreen
 */
Minitel.MinitelScreen = class {
    /**
     * @param {HTMLCanvasElement} canvas
     * @param {} color
     * @param {HTMLAudioElement} bip
     */
    constructor(canvas, color, bip=null) {
        const grid = { cols: Minitel.columns, rows: Minitel.rows }
        const char = { width: Minitel.charWidth, height: Minitel.charHeight }

        // Resize canvas based on Minitel characteristics
        canvas.width = char.width * grid.cols
        canvas.height = char.height * grid.rows

        /**
         * Should we show colors or black and white?
         * @param {boolean}
         */
        this.color = color ? true : false

        /**
         * The Visual Display Unit
         * @member {Minitel.VDU}
         * @private
         */
        this.vdu = new Minitel.VDU(
            grid,
            char,
            canvas,
            this.color ? Minitel.colors : Minitel.greys
        )

        /**
         * The decoder
         * @member {MinitelDecoder}
         * @private
         */
        this.decoder = new Minitel.Decoder(this.vdu, null, null, bip)

        /**
         * The queue
         * @member {number[]}
         * @private
         */
        this.queue = []

        /**
         * How many bytes are sent to the page memory each refresh time
         * @member {number}
         */
        this.chunkSize = 0

        /**
         * The ID value of the timer used to refresh the Minitel screen
         * @param {number}
         */
        this.timer = undefined

        /**
         * Should the cursor position be shown?
         * @param {boolean}
         */
        this.cursorShown = false

        this.initRefresh(Minitel.B1200, 25)
    }

    /**
     * Initializes the timer used to refresh the Minitel screen
     *
     * @param {number} bandwidth Bandwidth in bits per second
     * @param {number} rate Refresh rate in milliseconds
     */
    initRefresh(bandwidth, rate) {
        // Stop any existing timer
        if(this.timer) window.clearInterval(this.timer)

        // Minitel uses 9 bits for each code (7 bit of data, 1 parity bit and
        // 1 stop bit)
        this.chunkSize = bandwidth / 9 / (1000 / rate)
        this.timer = window.setInterval(() => this.sendChunk(), rate)
    }

    /**
     * Push values in the queue for future send
     * @param {number[]} items Values to send
     */
    send(items) {
        this.queue = this.queue.concat(items)
    }

    /**
     * Directly send values to the page memory, bypassing the refresh rate.
     * @param {number[]} items Values to send
     */
    directSend(items) {
        this.queue = []
        this.decoder.decodeList(items)
    }

    /**
     * Generate a thumbnail of the current display.
     * @param {number} width Width of the thumbnail
     * @param {number} height Height of the thumbnail
     */
    generateThumbnail(width, height) {
        return this.vdu.generateThumbnail(width, height)
    }

    /**
     * Send a chunk of the queue to the page memory. This method is called by
     * the timer.
     * @private
     */
    sendChunk() {
        // Nothing to do?
        if(this.queue.length === 0) return

        const chunk = this.queue.slice(0, this.chunkSize)
        this.queue = this.queue.slice(this.chunkSize)
        this.decoder.decodeList(chunk)
    }
}
