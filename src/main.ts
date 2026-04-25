import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import './styles/rm-theme.css'
import './styles/rm-hud.css'
import './styles/tactical-layout.css'
import App from './App.vue'
import { router } from './router'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
