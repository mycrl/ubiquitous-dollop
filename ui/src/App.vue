<template>
    <div class="Main">
        <Devices
            :devices="selectDevices"
            @device="selectDevice"
        />
        
        <!--
        连接报告
        -->
        <div 
            class="Report box"
            v-show=" report.state === 'succeeded' "
            :style="{
                backgroundColor: connecting && mousemove.sleep ? 'rgba(0, 0, 0, 0)' : null,
                border: connecting && mousemove.sleep ? 'rgba(0, 0, 0, 0)' : '1px solid #ddd',
                color: connecting && mousemove.sleep ? '#999' : null,
            }"
        >
            
            <!--
            指示灯
            -->
            <div class="indicator_light">
                <div 
                    class="light"
                    :style="{ backgroundColor: ({
                        'checking': '#0c9cf8',
                        'connected': '#00b150',
                        'disconnected': '#f00222',
                        'closed': '#999',
                        'failed': '#f00222'
                    })[connectionState] }"
                ></div>
                <span>{{ ({
                        'checking': '连接中...',
                        'connected': '连接完成',
                        'disconnected': '连接断开',
                        'closed': '连接关闭',
                        'failed': '连接失败'
                    })[connectionState] }}
                </span>
            </div>
            
            <!--
            信息列表
            -->
            <div class="info">
                <p>发送: {{ report.packetsSent }} byte/s</p>
                <p>接收: {{ report.packetsReceived }} byte/s</p>
                <p>延迟: {{ report.totalRoundTripTime }} ms</p>
            </div>
        </div>
        
        <!--
        连接窗口
        用户连接对端
        -->
        <div 
            class="Connection box"
            v-show="!connecting"
        >
            <input 
                type="text" 
                placeholder="对方号码"
                v-model="peer"
            />
            <div>
                <div>
                    <span>你的号码:</span>
                    <p>{{ code }}</p>
                </div>
                <button @click="connection">
                    连接
                </button>
            </div>
        </div>
        
        <!--
        输入管理
        -->
        <div 
            class="Input box"
            v-show="!connecting || !mousemove.sleep"
            :style="{ bottom: !(outputs.uri || player.canplay)? '20px' : '80px' }"
        >
            <div class="option">
                
                <!--
                音频输入设备列表
                -->
                <p class="title">音频</p>
                
                <!--
                添加音频输入设备
                -->
                <div 
                    class="add hover"
                    @click="addAudioInputDevice"
                >
                    <img src="@/assets/addto.svg"/>
                </div>
                <p 
                    class="none" 
                    v-show="inputs.audios.length == 0"
                >没有音频输入设备...</p>
                <AudioDevice 
                    class="audio" 
                    v-for="audio of inputs.audios"
                    :key="audio.id"
                    :device="audio"
                    :canplay="false"
                    @volume="volumeChange"
                />
            </div>
            <div 
                class="option"
                v-show="!(outputs.uri || player.canplay)"
            >
                
                <!--
                视频输入设备
                -->
                <p 
                   class="title" 
                   style="margin-top: 10px;"
                >视频</p>
                <VideoDevice 
                    class="video hover"
                    @add="addVideoInputDevice"
                />
            </div>
        </div>
        
        <!--
        输出管理
        -->
        <div 
            class="Output box"
            v-show=" outputs.audios.length > 0 && !mousemove.sleep "
        >
            
            <!--
            音频输入设备列表
            -->
            <AudioDevice 
                class="audio" 
                v-for="audio of outputs.audios"
                :key="audio.deviceId"
                :device="audio"
                :canplay="true"
            />
        </div>
        
        <!--
        播放器
        -->
        <div 
            class="Player"
            :style="{
                opacity: outputs.uri || player.canplay ? 1 : 0     
            }"
        >
            <video
                muted
                autoplay
                controls
                :src="outputs.uri"
                @canplay="canplay"
                ref="player"
            />
        </div>
    </div>
</template>

<script>
    import Devices from '@/components/Devices'
    import AudioDevice from '@/components/AudioDevice'
    import VideoDevice from '@/components/VideoDevice'
    import Streamer, { Code, Device, DeviceKind } from '@/streamer'
    
    const code = Code.build()
    const streamer = new Streamer({
        code,
        signaling: `wss://psyai.net/signaling/${code}?dev`,
        rtc: {
            iceTransportPolicy: 'all',
            iceServers: [{
                urls: 'turn:71.131.210.117',
                username: 'psyai',
                credential: 'psyai',
                credentialType: 'password'
            }]
        }
    })
    
    export default {
        components: {
            AudioDevice,
            VideoDevice,
            Devices
        },
        data() {
            return {
                code,
                report: {},
                peer: null,
                connecting: false,
                connectionState: 'new',
                selectDevices: [],
                inputs: {
                    audios: streamer.inputs,
                },
                outputs: {
                    audios: streamer.outputs,
                    uri: null
                },
                mousemove: {
                    rc: 0,
                    sleep: false,
                },
                player: {
                    canplay: false
                }
            }
        },
        methods: {
            
            /**
             * 视频输入
             * 
             * @returns {Promise<void>}
             * @private
             */
            async addVideoInputDevice(url) {
                this.outputs.uri = url
            },
            
            /**
             * 连接音输入设备
             * 
             * @returns {Promise<void>}
             * @private
             */
            async addAudioInputDevice() {
                this.selectDevices = (await Device.getInputDevices(DeviceKind.Audio))
                    .map(device => Object.assign(device, { 
                        name: Device.getDeviceName(device.label)
                    }))
            },
            
            /**
             * 选择设备
             * 弹窗选择设备
             * 
             * @param {Device} device
             * @returns {Promise<void>}
             * @private
             */
            async selectDevice(device) {
                streamer.addDevice(device)
                this.selectDevices = []
            },
            
            /**
             * 建立rtc连接
             * 
             * @returns {Promise<void>}
             * @private
             */
            async connection() {
                streamer.to = this.peer
                await streamer.launch()
            },
            
            /**
             * 主循环
             * 
             * @returns {Promise<void>}
             * @private
             */
            async pool() {
                for (const [_, report] of await streamer.stats) {
                    if (report.type === 'candidate-pair' && report.nominated) {
                        this.report = report 
                    }
                }
                
                if (!this.mousemove.sleep) {
                    if (this.mousemove.rc < 5) {
                        this.mousemove.rc += 1
                    } else {
                        this.mousemove.sleep = true
                    }   
                }
            },
            
            /**
             * 处理连接状态变更信息
             * 
             * @returns {void}
             * @private
             */
            onStateChange(state) {
                this.connectionState = state === 'failed' ? 'new' : state
                this.connecting = state != 'new'
            },
            
            /**
             * 处理鼠标移动
             * 
             * @returns {void}
             * @private
             */
            onMousemove() {
                this.mousemove.sleep = false
                this.mousemove.rc = 0
            },
            
            /**
             * 处理音量变更
             * 
             * @param {number} [volume]
             * @param {string} [device]
             * @returns {void}
             * @private
             */
            volumeChange({ volume, device }) {
                streamer.setVolume(device, volume)
            },
            
            /**
             * 监听远程音量变更
             * 
             * @param {number} [volume]
             * @param {string} [device]
             * @returns {void}
             * @private
             */
            onRemoteVolume({ volume, device }) {
                for (const device of this.outputs.audios) {
                    if (device.id === device) {
                        device.volume = volume
                    }
                }
            },
            
            canplay() {
                this.player.canplay = true
            }
        },
        mounted() {
            document.addEventListener('mousemove', this.onMousemove.bind(this), false)
            streamer.on('stateChange', this.onStateChange.bind(this))
            streamer.on('volume', this.onRemoteVolume.bind(this))
            setInterval(this.pool.bind(this), 1000)
            streamer.player = this.$refs.player
        }
    }
</script>

<style scoped>
    .Main {
        position: fixed;
        width: 100%;
        height: 100%;
        top: 0;
        left: 0;
    }
    
    .Connection {
        position: absolute;
        top: 20px;
        left: 20px;
        width: 200px;
        z-index: 3;
    }
    
    .Connection input {
        width: 100%;
        line-height: 40px;
        border: 1px solid #eee;
        font-size: 13px;
        border-radius: 5px;
        text-indent: 15px;
    }
    
    .Connection > div {
        margin-top: 10px;
        display: flex;
    }
    
    .Connection > div > * {
        flex: 1;
    }
    
    .Connection div button {
        line-height: 35px;
        background-color: #fff;
        border: 1px solid #6D84DC;
        color: #6D84DC;
        border-radius: 5px;
    }
    
    .Connection div span {
        font-size: 0.8rem;
        color: #999;
    } 
    
    .Connection div p {
        font-weight: bold;
        color: #555;
        margin-top: 5px;
    }
    
    .Input {
        position: absolute;
        width: 200px;
        left: 20px;
        z-index: 2;
    }
    
    .Input .option .title {
        margin-bottom: 10px;
    }
    
    .Input .option .add {
        position: absolute;
        top: 13px;
        right: 15px;
    }
    
    .Input .option .add img {
        width: 25px;
    }
    
    .Input .option .audio {
        padding: 10px 0;
    }
    
    .Input .option .none {
        line-height: 40px;
        color: #999;
    }
    
    .Input .option .video {
        margin-top: 10px;
    }
    
    .Output {
        position: absolute;
        width: 200px;
        bottom: 80px;
        right: 20px;
        z-index: 2;
    }
    
    .Player {
        z-index: 1;
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: #000;
    }
    
    .Player video {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
    }
    
    .Report {
        position: absolute;
        top: 20px;
        left: 20px;
        width: 200px;
        z-index: 3;
    }
    
    .Report .indicator_light {
        margin-bottom: 10px;
    }
    
    .Report .indicator_light .light {
        width: 15px;
        height: 15px;
        border-radius: 50%;
        float: left;
    }
    
    .Report .indicator_light span {
        margin-left: 10px;
    }
</style>

<style>
    * {
        margin: 0;
        padding: 0;
        font-size: 12px;
        font-family: FMBook, FMMedium, Microsoft YaHei, sans-serif;
    }
    
    body {
        background-color: #efefef;
    }
    
    a {
        text-decoration: none;
        vertical-align: baseline;
    }
    
    input, select {
        outline: none;
    }
    
    button, .hover {
        cursor: pointer;
        outline: none;
    }
    
    .v-shadow {
        box-shadow: 0px 1px 2px 0px rgba(60, 64, 67, 0.3), 
            0px 2px 6px 2px rgba(60, 64, 67, 0.15);
        background-color: #fff;
    }
    
    .scroll::-webkit-scrollbar { width: 0 !important; }
    .scroll { overflow: -moz-scrollbars-none; }
    .scroll { -ms-overflow-style: none; }
    .uncopy {
        user-select: none;
    }
    
    .disable {
        filter: grayscale(100%);
    }
    
    .box {
        background-color: #fff;
        border-radius: 7px;
        border: 1px solid #ddd;
        padding: 15px;
    }
</style>
