/* eslint-disable */
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

declare global {
    interface Window {
        __TAURI_INVOKE__<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
    }
}

// Function avoids 'window not defined' in SSR
const invoke = () => window.__TAURI_INVOKE__;

export function appReady() {
    return invoke()<null>("app_ready")
}

export function resetSpacedrive() {
    return invoke()<null>("reset_spacedrive")
}

export function openLogsDir() {
    return invoke()<null>("open_logs_dir")
}

export function openFilePath(library: string, id: number) {
    return invoke()<OpenFilePathResult>("open_file_path", { library,id })
}

export function getFilePathOpenWithApps(library: string, id: number) {
    return invoke()<OpenWithApplication[]>("get_file_path_open_with_apps", { library,id })
}

export function openFilePathWith(library: string, id: number, withUrl: string) {
    return invoke()<null>("open_file_path_with", { library,id,withUrl })
}

export type OpenWithApplication = { name: string; url: string }
export type OpenFilePathResult = { t: "NoLibrary" } | { t: "NoFile" } | { t: "OpenError"; c: string } | { t: "AllGood" }
