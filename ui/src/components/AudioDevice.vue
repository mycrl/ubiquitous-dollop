<template>
    
    <!--
    音频设备
    -->
    <div class="AudioDevice">
        
        <!--
        设备信息
        -->
        <div class="info">
            <img 
                class="icon" 
                src="@/assets/audio.svg"
            />
            
            <!--
            设备名称
            -->
            <span>{{ device.name }}</span>
        </div>
        
        <!--
        音量控制
        -->
        <div 
            class="volume hover"
            @click="e => setVolume(e)"
            ref="volume"
        >
            
            <!--
            音量滑块
            -->
            <div 
                class="slider"
                :style="{ width: device.volume + '%' }"
            ></div>
            
            <!--
            电平
            -->
            <div 
                class="level"
                :style="{ width: level + '%' }"
            ></div>
        </div>
        
        <!--
        音频播放器
        -->
        <audio
            autoplay
            v-if="canplay"
            :srcObject="device.stream"
            ref="player"
        />
    </div>
</template>

<script>
    export default {
        name: 'AudioDevice',
        props: {
            device: Object,
            canplay: {
                type: Boolean,
                default: false
            }
        },
        data() {
            return {
                level: 0,
                buffer: null,
                analyserNode: null,
                loop: null,
            }  
        },
        methods: {
            
            /**
             * 创建音频处理器
             * 用来处理音频数据
             * 
             * @returns {void}
             * @private
             */
            createAudioProcess() {
                const ctx = new AudioContext()
                const mediaStreamAudioSourceNode = ctx.createMediaStreamSource(this.device.stream)
                this.analyserNode = ctx.createAnalyser()
                mediaStreamAudioSourceNode.connect(this.analyserNode)
                this.buffer = new Float32Array(this.analyserNode.fftSize)
                this.loop = requestAnimationFrame(this.processAudio.bind(this))
            },
            
            /**
             * 处理音频数据
             * 获取当前音频轨道的响度
             * 
             * @returns {void}
             * @private
             */
            processAudio() {
                if (!this.device.stream.active) {
                    this.stop()
                }
                
                this.analyserNode.getFloatTimeDomainData(this.buffer)
                
                let sumSquares = 0.0
                for (const amplitude of this.buffer) {
                    sumSquares += amplitude * amplitude
                }
                
                const level = Math.floor(sumSquares) / 100 * 2
                this.level = this.device.volume * (level > 1 ? 1 : level)
                this.loop = requestAnimationFrame(this.processAudio.bind(this))
            },
            
            /**
             * 停止处理
             * 关闭主循环
             * 
             * @returns {void}
             * @private
             */
            stop() {
                if (this.loop != null) {
                    cancelAnimationFrame(this.loop)   
                }
            },
            
            /**
             * 调整音量
             * 
             * @returns {void}
             * @private
             */
            setVolume({ offsetX }) {
                const level = Math.floor((offsetX / this.$refs.volume.clientWidth) * 100)
                this.device.volume = level > 100 ? 100 : level
                this.$refs.player.volume = this.device.volume / 100
            }
        },
        mounted() {
            this.stop()
            this.createAudioProcess()
        }
    }
</script>

<style scoped>
    .AudioDevice .info .icon {
        height: 14px;
    }
    
    .AudioDevice .info span {
        color: #999;
        margin-left: 10px;
        position: relative;
        top: -3px;
        font-size: 10px;
        font-family: monospace;
    }
    
    .AudioDevice .volume {
        height: 5px;
        background-color: #eee;
        margin-top: 7px;
        position: relative;
        border-radius: 5px;
    }
    
    .AudioDevice .volume .slider {
        position: absolute;
        height: 100%;
        top: 0;
        left: 0;
        background-color: #0c9cf8;
        border-radius: 5px;
    }
    
    .AudioDevice .volume .level {
        position: absolute;
        height: 100%;
        top: 0;
        left: 0;
        background-color: #f00222;
        border-radius: 5px;
    }
</style>