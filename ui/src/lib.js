// Wrappers autour des commandes Tauri (source unique côté frontend).
import { invoke } from "@tauri-apps/api/core";

export const serverStatus = () => invoke("server_status");
export const endpointInfo = () => invoke("endpoint_info");
export const appBuild = () => invoke("app_build");
export const startServer = () => invoke("start_server");
export const stopServer = () => invoke("stop_server");
export const openWindow = (view) => invoke("open_window", { view });

export const dictList = () => invoke("dict_list");
export const dictSave = (dict) => invoke("dict_save", { dict });
export const dictAddTerm = (canonical, alias) => invoke("dict_add_term", { canonical, alias });

export const journalList = (limit) => invoke("journal_list", { limit });
export const journalClear = () => invoke("journal_clear");

export const statsSummary = () => invoke("stats_summary");

export const configGet = () => invoke("config_get");
export const configSave = (config) => invoke("config_save", { config });
