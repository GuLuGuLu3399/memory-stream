import { createApp } from "vue";
import { createPinia } from "pinia";
import "./style.css";
import "@memory-stream/ui-shared/styles/transitions.css";
import App from "./App.vue";

createApp(App).use(createPinia()).mount("#app");
