// Wrappers autour des commandes Tauri (source unique côté frontend).
import { invoke } from "@tauri-apps/api/core";

export const serverStatus = () => invoke("server_status");
export const endpointInfo = () => invoke("endpoint_info");
export const startServer = () => invoke("start_server");
export const stopServer = () => invoke("stop_server");
export const openWindow = (view) => invoke("open_window", { view });
