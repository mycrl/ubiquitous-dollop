'use strict'

import EventEmitter from './events'

/**
 * 设备类型
 * 
 * @readonly
 */
export const DeviceKind = {
    Audio: 'audioinput',
    Video: 'videoinput',
}

/**
 * 默认请求媒体类型配置
 * 
 * @readonly
 */
export const DefaultConstraints = {
    video: true,
    audio: true,
}

/**
 * 系统设备ID
 * 
 * @readonly
 */
export const SystemDeviceIds = [
    'default', 
    'communications',
]

/**
 * 号码
 * 
 * @class
 */
export class Code {
    
    /**
     * @retutns {string}
     * @public
     */
    static build() {
        const buf = new Array(6).fill(0)
        const rand = buf.map(() => Math.floor(Math.random() * 10))
        return rand.join('')
    }
}

/**
 * 设备
 * 
 * @class
 */
export class Device {
    
    /**
     * @param {string} label
     * @retutns {string}
     * @public
     */
    static getDeviceName(label) {
        const name = label.replace(/\(([0-9]|[a-z]){4}:([0-9]|[a-z]){4}\)/g, '')
        return name.includes('(') ? name.match(/(?<=\()(.+?)(?=\))/g)[0] : name
    }
    
    /**
     * @param {object} constraints
     * @returns {Promise<MediaStream>}
     * @public
     */
    static async getMedia(constraints = DefaultConstraints) {
        return await navigator.mediaDevices.getUserMedia(constraints)
    }

    /**
     * @param {DeviceKind} kind
     * @returns {Promise<Device[]>}
     * @public
     */
    static async getInputDevices(kind) {
        return (await navigator.mediaDevices.enumerateDevices())
            .filter(device => device.kind === kind)
            .filter(({ deviceId }) => !SystemDeviceIds.includes(deviceId))
    }
}

/**
 * 流
 * 
 * @class
 */
export class Stream extends EventEmitter {
    
    /**
     * @param {Streamer} streamer
     * @param {MediaStream} stream
     * @constructor
     */
    constructor(streamer, track) {
        super()
        this._player = null
        this._track = track
        this._streamer = streamer
        this._stream = new MediaStream()
        this._stream.addTrack(this._track)
    }
    
    /**
     * 
     */
    get track() {
        return this._track
    }
    
    /**
     * 
     */
    get stream() {
        return this._stream
    }
    
    /**
     * 
     */
    get kind() {
        return this._track.kind
    }
    
    /**
     * 
     */
    get id() {
        return this._track.id
    }
    
    /**
     * 
     */
    get name() {
        return this._track.contentHint
    }
    
    /**
     * 
     */
    get volume() {
        return 50
    }
    
    /**
     * @param {string} id
     * @param {number} volume
     * @returns {void}
     * @public
     */
    set volume(volume) {
        
    }
    
    /**
     * 
     */
    set player(player) {
        this._player = player
        this._player.srcObject = this._stream
        return true
    }
    
    /**
     * 
     */
    remove() {
        
    }
}

/**
 * 媒体流
 * 
 * @class
 */
export default class Streamer extends EventEmitter {
    
    /**
     * @param {object} options
     * @constructor
     */ 
    constructor(options) {
        super()
        this._to = null
        this._state = {}
        this._inputs = []
        this._outputs = []
        this._options = options
        this._ws = new WebSocket(this._options.signaling)
        this._connection = new RTCPeerConnection(this._options.rtc)
        this._init()
    }

    /**
     * @returns {void}
     * @private
     */ 
    _init() {
        this._ws.onmessage = this._onMessage.bind(this)
        this._connection.addEventListener('track', this._onTrack.bind(this))
        this._connection.addEventListener('icecandidate', this._onIceCandidate.bind(this))
        this._connection.addEventListener('connectionstatechange', this._onConnectionstatechange.bind(this))
        this._connection.addEventListener('negotiationneeded', this._onNegotiationneeded.bind(this))
    }
    
    /**
     * send data to server.
     * @param {string} type
     * @param {any} payload
     * @returns {void}
     * @private
     */
    _send(type, payload) {
        const to = this._to
        const from = this._options.code
        this._ws.send(JSON.stringify({ from, to, type, payload }))
    }
    
    /**
     * @returns {void}
     * @private
     */ 
    async _onNegotiationneeded() {
        if (this._connection.connectionState == 'new') return
        await this._connection.setLocalDescription()
        this._send('offer', this._connection.localDescription.sdp)
    }
    
    /**
     * @param {Event} event
     * @returns {void}
     * @private
     */ 
    _onConnectionstatechange(_event) {
        this.emit('stateChange', this._connection.connectionState)
    }
    
    /**
     * 
     */
    _addPlayerTrack(track) {
        if (this._player.srcObject == null)
            this._player.srcObject = new MediaStream()
        this._player.srcObject.addTrack(track)
    }
    
    /**
     * @param {MediaSteamTrack} track
     * @returns {void}
     * @private
     */ 
    _onTrack({ track }) {
        track.contentHint == '' ? 
            this._addPlayerTrack(track) :
            this._addRemoteTrack(track)
    }
    
    /**
     * @param {RTCIceCandidate} [candidate]
     * @returns {void}
     * @private
     */ 
    _onIceCandidate({ candidate }) {
        this._send('candidate', candidate)
    }
    
    /**
     * handle message.
     * @param {Event} event
     * @returns {void}
     * @private
     */
    _onMessage({ data }) {
        const { from, type, payload } = JSON.parse(data)
        if (type == 'candidate') this._onCandidate(payload)
        if (type == 'connect') this._onConnect(from)
        if (type == 'answer') this._onAnswer(payload)
        if (type == 'offer') this._onOffer(payload)
        if (type == 'state') this._onState(payload)
    }
    
    /**
     * @param {string} from
     * @returns {void}
     * @public
     */ 
    _onConnect(from) {
        this._to = from
    }
    
    /**
     * @param {RTCIceCandidate} candidate
     * @returns {Promise<void>}
     * @public
     */ 
    _onCandidate(candidate) {
        this._connection.addIceCandidate(candidate)
    }
    
    /**
     * @param {RTCSessionDescriptionInit} answer
     * @returns {Promise<void>}
     * @public
     */ 
    _onAnswer(sdp) {
        this._connection.setRemoteDescription({ type: 'answer', sdp })
    }
    
    /**
     * @param {RTCSessionDescriptionInit} answer
     * @returns {Promise<RTCSessionDescriptionInit>}
     * @public
     */ 
    async _onOffer(sdp) {
        this._connection.setRemoteDescription({ type: 'offer', sdp })
        await this._connection.setLocalDescription()
        this._send('answer', this._connection.localDescription.sdp)
    }
    
    /**
     * @param {object} state
     * @returns {void}
     * @private
     */
    _onState(state) {
        this._state = state
    }
    
    /**
     * 
     */
    _stateChange() {
        
    }
    
    /**
     * 
     */
    _addRemoteTrack(track) {
        this._outputs.push(new Stream(this, track))
    }
    
    /**
     * 
     */
    _addLocalTrack(track) {
        const stream = new Stream(this, track)
        this._connection.addTrack(track, stream.stream)
        this._inputs.push(stream)
    }
    
    /**
     * 
     */
    _capturePlayerStream() {
        const stream = this._player.captureStream()
        for (const track of stream.getTracks())
            this._connection.addTrack(track, stream)
    }
    
    /**
     * @param {string} to
     * @returns {void}
     * @public
     */
    set to(to) {
        this._to = to
    }
    
    /**
     * @returns {Array<Device>}
     * @public
     */
    get inputs() {
        return this._inputs
    }
    
    /**
     * 
     */
    get outputs() {
        return this._outputs
    }
    
    /**
     * @returns {RTCStateReport}
     * @public
     */
    get stats() {
        return this._connection.getStats()
    }
    
    /**
     * @param {MediaStream} stream
     * @returns {void}
     * @public
     */
    set player(player) {
        this._player = player
    }
    
    /**
     * @param {function} handler
     * @returns {Device}
     * @public
     */
    async addDevice(device) {
        const stream = await Device.getMedia({audio: device, video: false})
        stream.getTracks()[0].contentHint = device.name
        this._addLocalTrack(stream.getTracks()[0])
    }
    
    /**
     * @returns {Promise<void>}
     * @public
     */ 
    async launch() {
        this._send('connect')
        this._capturePlayerStream()
        const offer = await this._connection.createOffer()
        await this._connection.setLocalDescription(offer)
        this._send('offer', this._connection.localDescription.sdp)
    }
}