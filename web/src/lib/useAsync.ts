import { onMounted, ref, type Ref } from 'vue'
import { errorMessage } from './api'

export interface AsyncResource<T> {
  data: Ref<T | null>
  loading: Ref<boolean>
  error: Ref<string>
  reload: (silent?: boolean) => Promise<void>
}

/** Loads on mount; `reload(true)` refreshes without flashing the skeleton. */
export function useAsync<T>(loader: () => Promise<T>, immediate = true): AsyncResource<T> {
  const data = ref<T | null>(null) as Ref<T | null>
  const loading = ref(immediate)
  const error = ref('')
  let ticket = 0

  async function reload(silent = false) {
    const mine = ++ticket
    if (!silent) loading.value = true
    error.value = ''
    try {
      const result = await loader()
      if (mine === ticket) data.value = result
    } catch (e) {
      if (mine === ticket) error.value = errorMessage(e, '加载失败')
    } finally {
      if (mine === ticket) loading.value = false
    }
  }

  if (immediate) onMounted(() => reload())
  return { data, loading, error, reload }
}

/** Wraps a mutation with a busy flag and toast-on-error. */
export function useAction() {
  const busy = ref(false)
  async function run<T>(fn: () => Promise<T>, onError: (message: string) => void): Promise<T | undefined> {
    if (busy.value) return undefined
    busy.value = true
    try {
      return await fn()
    } catch (e) {
      onError(errorMessage(e))
      return undefined
    } finally {
      busy.value = false
    }
  }
  return { busy, run }
}
