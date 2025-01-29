"use strict"
/**
 * @file key-simulator
 * @author Frédéric BISSON <zigazou@free.fr>
 * @version 1.0
 *
 * This class aims to simulate keyboard click sounds.
 *
 */

/**
 * @class KeySimulator
 */
class KeySimulator {
    /**
     * @param {HTMLAudioElement} keyDownSound Sound when key is pressed.
     * @param {HTMLAudioElement} keyUpSound Sound when key is released.
     */
    constructor(keyDownSound, keyUpSound) {
        this.keyPressed = new Set()

        this.concurrentSound = 5
        this.downStack = new Array(this.concurrentSound)
        this.upStack = keyUpSound === undefined
                     ? undefined
                     : new Array(this.concurrentSound)

        this.currentDownOffset = 0
        this.currentUpOffset = 0

        range(this.concurrentSound).forEach(i => {
            this.downStack[i] = keyDownSound.cloneNode(true)
            if(this.upStack !== undefined) {
                this.upStack[i] = keyUpSound.cloneNode(true)
            }
        })
    }

    /**
     * Play the press sound
     */
    pressKey(keyCode) {
        if(this.upStack !== undefined) {
            if(this.keyPressed.has(keyCode)) return
            this.keyPressed.add(keyCode)
        }

        this.downStack[this.currentDownOffset].currentTime = 0
        this.downStack[this.currentDownOffset].play()

        this.currentDownOffset = (this.currentDownOffset + 1)
                               % this.concurrentSound
    }

    /**
     * Play the release sound
     */
    releaseKey(keyCode) {
        if(this.upStack === undefined) return
        if(!this.keyPressed.has(keyCode)) return

        this.keyPressed.delete(keyCode)

        this.upStack[this.currentDownOffset].currentTime = 0
        this.upStack[this.currentDownOffset].play()

        this.currentUpOffset = (this.currentUpOffset + 1)
                             % this.concurrentSound
    }
}
