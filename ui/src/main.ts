import "./styles/global.css";
import App from "./app/App.svelte";

const app = new App({
  target: document.getElementById("app") as HTMLElement
});

export default app;
