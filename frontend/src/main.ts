import App from '@/App.vue'
import router from '@/bridge/router'
import { AllCommunityModule, ModuleRegistry } from "ag-grid-community";
ModuleRegistry.registerModules([AllCommunityModule]);

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
