import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

import { library } from '@fortawesome/fontawesome-svg-core'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { faGear, faTowerBroadcast } from '@fortawesome/free-solid-svg-icons'

library.add(...[
    faGear,
    faTowerBroadcast,
])

createApp(App)
    .component('icon', FontAwesomeIcon)
    .mount('#app')
