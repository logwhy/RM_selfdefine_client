export function readFromStorage<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key)
    if (!raw) {
      return null
    }
    return JSON.parse(raw) as T
  } catch (error) {
    console.warn(`failed to read localStorage key: ${key}`, error)
    return null
  }
}

export function writeToStorage<T>(key: string, value: T): boolean {
  try {
    localStorage.setItem(key, JSON.stringify(value))
    return true
  } catch (error) {
    console.warn(`failed to write localStorage key: ${key}`, error)
    return false
  }
}
