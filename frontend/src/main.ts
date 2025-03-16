import { createApp } from "vue";
import "@/assets/index.css";
import App from "@/App.vue";
import { createMemoryHistory, createRouter } from "vue-router";

import { APN, IPCC } from "./views";

const routes = [
  {
    path: "/ipcc",
    name: "IPCC",
    component: IPCC,
  },
  {
    path: "/apn",
    name: "APN",
    component: APN,
  },
];

const router = createRouter({
  history: createMemoryHistory(import.meta.env.BASE_URL),
  routes,
});

createApp(App).use(router).mount("#app");
