<template>
    <div class="Connection">
        <div class="Code">
            <div class="Head">
                <div class="Settings">
                    <icon 
                        class="icon hover" 
                        :icon="['fas', 'gear']"
                    />
                </div>
            </div>
            <div class="Title">
                <icon 
                    class="icon" 
                    :icon="['fas', 'tower-broadcast']"
                />
                <h3>Connecting</h3>
                <p>
                    Please share your code 
                    <span>8899</span> 
                    with the other party
                </p>
            </div>
            <div 
                class="PeerCode"
                ref="codesNode"
            >
                <div>
                    <input 
                        type="text" 
                        v-for="(value, index) of code"
                        @input="event => onInput(event.target.value, index)"
                        :class="value != null ? 'inputed' : null"
                        :value="code[index]"
                    />
                </div>
            </div>
            <button id="Connect">Connect</button>
            <div class="Help">
                <span>unable to connect? Please check whether the other party's code is correct.</span>
            </div>
        </div>
        <div class="Scan">
            <div class="Animation">
                <div class="circle"/>
                <div class="circle"/>
                <div class="circle"/>
                <div class="circle"/>
            </div>
            <div class="Nodes">
                <div class="Node">
                </div>
            </div>
            <div class="Help">
                <span>Scanning for other users on LAN...</span>
            </div>
        </div>
    </div>
</template>

<script setup>
    import { ref } from 'vue'
    
    let code = ref(new Array(4).fill(null))
    let codesNode = ref()

    function onInput(value, index) {
        code.value[index] = value
        if (value.length == 0) {
            if (index > 0) {
                codesNode.value.querySelectorAll('input')[index - 1].focus()
            }
        } else {
            if (index < code.value.length - 1) {
                codesNode.value.querySelectorAll('input')[index + 1].focus()
            }
        }
    }
</script>

<style scoped>
    .Connection {
        background-color: #fff;
        height: 100vh;
        display: flex;
    }
    
    .Code {
        flex: 1;
        padding: 20px;
        text-align: center;
    }
    
    .Scan {
        flex: 2;
        background-color: #f3f4f6;
        border-radius: 70px 0 0 70px;
    }
    
    .Code .Head .Settings {
        text-align: left;
    }
    
    .Code .Title {
        margin-top: 12vh;
    }
    
    .Code .Title .icon {
        border: 1px solid #ddd;
        padding: 10px;
        color: #999;
        margin-bottom: 25px;
    }
    
    .Code .Title h3 {
        text-align: center;
        margin-bottom: 20px;
        font-weight: bold;
    }
    
    .Code .Title p {
        color: #626262;
        font-size: 0.8rem;
        margin-bottom: 25px;
    }
    
    .Code .Title span {
        color: #555;
        font-weight: bold;
    }
    
    .Code .PeerCode {
        display: table;
        width: 100%;
        margin-bottom: 20px;
    }
    
    .Code .PeerCode > div {
        margin: 0 auto;
        width: 246px;
    }
    
    .Code .PeerCode input {
        width: 50px;
        height: 50px;
        margin-right: 10px;
        border-radius: 5px;
        border: 2px solid #ddd;
        font-size: 30px;
        text-align: center;
        font-weight: bold;
    }
    
    .Code .PeerCode input:focus,
    .Code .PeerCode .inputed {
        border: 2px solid #165ef0;
    }
    
    .Code .PeerCode input:last-child {
        margin-right: 0;
    }
    
    .Code #Connect {
        width: 246px;
        height: 34px;
        border: 0;
        background-color: #165ef0;
        border-radius: 5px;
        color: #fff;
        font-size: 12px;
        margin-bottom: 20px;
    }
    
    .Code .Help {
        font-size: 12px;
        color: #999;
    }
    
    .Scan {
        position: relative;
    }
    
    .Scan .Animation {
        display: inline-block;
        position: absolute;
        top: 50%;
        left: 50%;
        width: 150px;
        height: 150px;
        margin-top: -187.5px;
        transform: rotate(-45deg) translate(-100px);
    }
    
    .Scan .Animation .circle {
        box-sizing: border-box;
        display: block;
        width: 100%;
        height: 100%;
        font-size: 21px;
        position: absolute;
        bottom: 0;
        left: 0;
        border-color: #165ef0;
        border-style: solid;
        border-width: 1em 1em 0 0;
        border-radius: 0 100% 0 0;
        opacity: 0;
        animation: ScanAnimation 3s infinite;
    }
    
    .Scan .Animation .circle:nth-of-type(1) {
        animation-delay: 800ms;
    }
    
    .Scan .Animation .circle:nth-of-type(2) {
        width: 5em;
        height: 5em;
        animation-delay: 400ms;
    }
    
    .Scan .Animation .circle:nth-of-type(3) {
        width: 3em;
        height: 3em;
    }
    
    .Scan .Animation .circle:nth-of-type(4) {
        width: 1em;
        height: 1em;
        opacity: 1;
        animation: none;
    }

    @keyframes ScanAnimation {
      0% { opacity: 0.8 }
      5% { opactiy: 1 }
      6% { opactiy: 0.1 }
      100% { opactiy: 0.1; }
    }
    
    .Scan .Help {
        position: absolute;
        width: 100%;
        bottom: 0;
        text-align: center;
        line-height: 100px;
        font-size: 0.8rem;
        color: #999;
    }
</style>