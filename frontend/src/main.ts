import App from '@/App.vue'
import router from '@/bridge/router'
import { AllCommunityModule, ModuleRegistry } from "ag-grid-community";
import "@fontsource/geist";
import '@/styles/global.css'

ModuleRegistry.registerModules([AllCommunityModule]);

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
