function meta(name: string, fallback: string): string {
  const value = document.querySelector(`meta[name="${name}"]`)?.getAttribute('content')?.trim() || ''
  // Placeholders stay unreplaced when the page is served by the Vite dev server.
  return !value || /^__[A-Z_]+__$/.test(value) ? fallback : value
}

export const site = {
  name: meta('app-name', 'notXboard'),
  version: meta('app-version', 'dev'),
  url: meta('app-url', ''),
}

export function brandMark(name: string): string {
  const letters = name.replace(/[^A-Za-z0-9\u4e00-\u9fa5]/g, '')
  const caps = letters.replace(/[^A-Z]/g, '')
  return (caps.length >= 2 ? caps.slice(0, 2) : letters.slice(0, 2)).toUpperCase() || 'NX'
}

export function setTitle(page?: string): void {
  document.title = page ? `${page} · ${site.name}` : site.name
}
