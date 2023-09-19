'use strict'

/**
 * this callback is handler.
 * @callback Handler
 * @param {[any]}
 * @returns {any}
 */

/**
 * @class EventEmitter
 */ 
export default class EventEmitter {
    
    /**
     * @constructor
     */ 
    constructor() {
        this._listener = {}
        this._events = {}
        this._index = -1
    }

    /**
     * @param {string} event - event name.
     * @private
     */ 
    _or_insert(event) {
        if (!this._events[event]) {
            this._events[event] = new Set()
        }
    }

    /**
     * bind eventer in events.
     * @param {string} event - event name.
     * @param {Handler} handler - event handler.
     * @returns {number}
     * @private
     */ 
    _bind(event, handler, once) {
        this._or_insert(event)
        this._index += 1
        let index = this._index
        this._events[event].add({ index, handler, once })
        this._listener[index] = event
        return index
    }

    /**
     * listen event.
     * @param {string} event - event name.
     * @param {Hnalder} handler
     * @returns {number}
     * @public
     */ 
    on(event, handle) {
        return this._bind(event, handle, false)
    }

    /**
     * once listen event.
     * @param {string} event - event name.
     * @param {Hnalder} handler
     * @returns {number}
     * @public
     */ 
    once(event, handle) {
        return this._bind(event, handle, true)
    }

    /**
     * remove event handler.
     * @param {string} event - event name.
     * @returns {void}
     * @public
     */ 
    remove(event) {
        delete this._events[event]
    }

    /**
     * remove listener.
     * @param {number} id - listener id.
     * @returns {void}
     * @public
     */ 
    pop(id) {
        let event = this._listener[id]
        let context = this._events[event][id]
        this._events[event].delete(context)
        delete this._listener[id]
    }

    /**
     * emit event.
     * @param {string} event - event name.
     * @param {[any]}
     * @returns {void}
     * @public
     */ 
    emit(event, ...argv) {
        if (!this._events[event]) 
            return undefined
         this._events[event]
             .forEach(e => {
                e.handler(...argv)
                e.once && this._events[event].delete(e)
            })
    }
}
