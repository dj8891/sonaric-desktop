import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'
import { open as tauriOpen } from '@tauri-apps/api/shell'

export function useTauri() {

  const open = (path: string, openWith?: string) => {
    if (window.__TAURI__) {
      tauriOpen(path)
    } else {
      window.open(path, openWith)
    }
  }
  return {
    invoke,
    listen,
    open
  }
}