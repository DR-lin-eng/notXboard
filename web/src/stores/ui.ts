import { defineStore } from 'pinia'

export type ToastKind = 'ok' | 'error' | 'info'
export interface Toast {
  id: number
  kind: ToastKind
  text: string
}
export interface DialogRequest {
  id: number
  kind: 'confirm' | 'prompt' | 'alert'
  title: string
  message?: string
  confirmText?: string
  danger?: boolean
  placeholder?: string
  value?: string
  multiline?: boolean
  resolve: (value: string | boolean | null) => void
}

let seq = 0

function initialTheme(): 'light' | 'dark' {
  return document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light'
}

export const useUi = defineStore('ui', {
  state: () => ({
    theme: initialTheme() as 'light' | 'dark',
    toasts: [] as Toast[],
    dialogs: [] as DialogRequest[],
    navOpen: false,
  }),
  actions: {
    setTheme(theme: 'light' | 'dark') {
      this.theme = theme
      document.documentElement.dataset.theme = theme
      try {
        localStorage.setItem('nx_theme', theme)
      } catch {
        // ignore
      }
    },
    toggleTheme() {
      this.setTheme(this.theme === 'dark' ? 'light' : 'dark')
    },
    toast(text: string, kind: ToastKind = 'info', timeout = 3600) {
      const id = ++seq
      this.toasts.push({ id, kind, text })
      window.setTimeout(() => {
        this.toasts = this.toasts.filter((item) => item.id !== id)
      }, timeout)
    },
    ok(text: string) {
      this.toast(text, 'ok')
    },
    error(text: string) {
      this.toast(text, 'error', 6000)
    },
    confirm(title: string, options: Partial<Omit<DialogRequest, 'id' | 'kind' | 'title' | 'resolve'>> = {}): Promise<boolean> {
      return new Promise((resolve) => {
        this.dialogs.push({ id: ++seq, kind: 'confirm', title, ...options, resolve: (v) => resolve(v === true) })
      })
    },
    alert(title: string, message = ''): Promise<void> {
      return new Promise((resolve) => {
        this.dialogs.push({ id: ++seq, kind: 'alert', title, message, resolve: () => resolve() })
      })
    },
    prompt(title: string, options: Partial<Omit<DialogRequest, 'id' | 'kind' | 'title' | 'resolve'>> = {}): Promise<string | null> {
      return new Promise((resolve) => {
        this.dialogs.push({
          id: ++seq,
          kind: 'prompt',
          title,
          ...options,
          resolve: (v) => resolve(typeof v === 'string' ? v : null),
        })
      })
    },
    closeDialog(id: number, value: string | boolean | null) {
      const dialog = this.dialogs.find((item) => item.id === id)
      this.dialogs = this.dialogs.filter((item) => item.id !== id)
      dialog?.resolve(value)
    },
  },
})
