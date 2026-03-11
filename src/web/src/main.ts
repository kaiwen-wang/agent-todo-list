import { createApp, Transition, TransitionGroup, h } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'

const app = createApp(App)

// Disable all Vue transitions globally — render children immediately with no animation
app.component('Transition', {
  setup(_: any, { slots }: any) {
    return () => slots.default?.()
  },
})
app.component('TransitionGroup', {
  setup(_: any, { slots }: any) {
    return () => slots.default?.()
  },
})

app.use(createPinia())
app.use(router)

app.mount('#app')
