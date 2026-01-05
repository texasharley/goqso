import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// Hide the HTML loading screen once React mounts
const loader = document.getElementById("app-loader");
if (loader) {
  loader.style.display = "none";
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
