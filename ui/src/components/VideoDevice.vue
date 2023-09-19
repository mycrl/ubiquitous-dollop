<template>
    
    
    <!--
    视频设备
    -->
    <div class="VideoDevice">
        
        <!--
        添加视频设备
        -->
        <div class="add">
            <img src="@/assets/video.svg"/>
            <input 
                type="file"
                class="hover"
                @change="selectFile"
            />
        </div>
    </div>
</template>

<script>
    export default {
        name: 'VideoDevice',
        methods: {
            selectFile({ target }) {
                this.$emit('add', URL.createObjectURL(
                    target.files[0]
                ))
            }
        }
    }
</script>

<style scoped>
    .VideoDevice {
        width: 100%;
        height: 200px;
        position: relative;
    }
    
    .VideoDevice .add {
        position: absolute;
        width: 100%;
        height: 100%;
        top: 0;
        left: 0;
        border-radius: 10px;
        text-align: center;
        background-color: #eee;
    }
    
    .VideoDevice .add img {
        width: 30px;
        margin-top: 80px;
    }
    
    .VideoDevice .add input {
        position: absolute;
        width: 100%;
        height: 100%;
        top: 0;
        left: 0;
        opacity: 0;
    }
    
    .VideoDevice video {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: #000;
        border-radius: 10px;
    }
</style>