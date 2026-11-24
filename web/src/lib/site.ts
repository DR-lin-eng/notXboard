function meta(name: string, fallback: string): string {
  const value = document.querySelector(`meta[name="${name}"]`)?.getAttribute('content')?.trim() || ''
  // Placeholders stay unreplaced when the page is served by the Vite dev server.
  return !value || /^__[A-Z_]+__$/.test(value) ? fallback : value
}

export const site = {
  name: meta('app-name', 'notXboard'),
  version: meta('app-version', 'dev'),
  url: meta('app-url', ''),
  /** `public` when the gateway serves the public data page at `/`; `app` for the signed-in application. */
  page: meta('app-page', 'app'),
}

export function brandMark(name: string): string {
  const letters = name.replace(/[^A-Za-z0-9\u4e00-\u9fa5]/g, '')
  const caps = letters.replace(/[^A-Z]/g, '')
  if (caps.length >= 2) return caps.slice(0, 2)
  // One inner capital, as in "notXboard", reads better as initial + capital than as the first two letters.
  const inner = letters.slice(1).match(/[A-Z]/)?.[0]
  return (inner ? letters[0] + inner : letters.slice(0, 2)).toUpperCase() || 'NX'
}

export function setTitle(page?: string): void {
  document.title = page ? `${page} · ${site.name}` : site.name
}
