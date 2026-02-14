import { createApp } from "vue";
import App from "./App.vue";
import "./styles.css";
import { applyFontName, getSavedFontName } from "./fontSettings";

applyFontName(getSavedFontName());

createApp(App).mount("#app");
