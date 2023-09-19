<template>
    
    <!--
    设备选择
    -->
    <div 
        class="Devices"
        v-show="Array.isArray(devices) && devices.length > 0"
    >

        <!--
        设备选择窗口
        -->
        <div class="box">
            <div class="title">
                选择设备
            </div>
            
            <!--
            设备列表
            -->
            <div class="values">
                <div 
                    class="option"
                    v-for="(device, i) of values"
                    :key="device.name"
                >
                    
                    <!--
                    设备选择框
                    -->
                    <input 
                        type="checkbox" 
                        class="hover"
                        v-model="device.lock"
                        @change="change(i)"
                    />
                    
                    <!--
                    设备名称
                    -->
                    <span>
                        {{ device.name }}
                    </span>
                </div>
            </div>
            
            <!--
            提交
            -->
            <div class="submit">
                <button 
                    @click="submit"
                    :style="{
                        border: `1px solid ${ disabled ? '#ddd' : '#6D84DC'}`,
                        color: disabled ? '#999' : '#6D84DC'            
                    }"
                >确定</button>
            </div>
        </div>
    </div>
</template>

<script>
    export default {
        name: 'Devices',
        props: {
            devices: {
                type: Array,
                default: () => []
            }
        },
        watch: {
            
            /**
             * 监听外部设备列表
             * copy到内部列表并加入临时选择器存储
             */
            devices(value) {
                this.values = value
                    .map(device => Object.assign(device, {
                        lock: false,
                    }))
            }  
        },
        data() {
            return {
                values: [],
                disabled: true
            }
        },
        methods: {

            /**
             * 处理设备选择事件
             * 单选只保留当前选择的项
             */
            change(index) {
                this.disabled = !this.values[index].lock
                if (this.disabled) {
                    return
                }
                
                this.values = this.values.map((device, offset) => {
                    device.lock = offset === index
                    return device
                })
            },
            
            /**
             * 提交选择的设备
             * 只有当按钮有效时才处理
             */
            submit() {
                if (this.disabled) {
                    return
                }
                
                const locks = this.values.filter(({ lock }) => lock)
                const device = locks[0]
                
                delete device.lock
                this.$emit('device', device)
            }
        }
    }
</script>

<style scoped>
    .Devices {
        position: fixed;
        width: 100%;
        height: 100%;
        z-index: 100;
        top: 0;
        left: 0;
        background-color: rgba(255, 255, 255, 0.8);
    }
    
    .Devices .box {
        width: 300px;
        margin: 0 auto;
        margin-top: 40vh;
    }
    
    .Devices .box .title {
        font-weight: 700;
        font-size: 14px;
        color: #555;
        line-height: 30px;
        margin-bottom: 20px;
    }
    
    .Devices .box .values .option span {
        color: #999;
        margin-left: 5px;
        position: relative;
        top: -2px;
    }
    
    .Devices .box .submit {
        width: 100%;
        display: table;
        margin-top: 20px;
    }
    
    .Devices .box .submit button {
        line-height: 35px;
        background-color: #fff;
        border-radius: 5px;
        width: 100px;
        float: right;
    }
</style>