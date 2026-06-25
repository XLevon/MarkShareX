import { ref, watch } from 'vue'

const isDark = ref(false)

// Initialize from localStorage or system preference
function initDarkMode() {
  const stored = localStorage.getItem('marksharex-dark-mode')
  if (stored !== null) {
    isDark.value = stored === 'true'
  } else {
    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  applyDarkMode()
}

function toggleDarkMode() {
  isDark.value = !isDark.value
  localStorage.setItem('marksharex-dark-mode', String(isDark.value))
  applyDarkMode()
}

function applyDarkMode() {
  if (isDark.value) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

// Listen for system preference changes
if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (localStorage.getItem('marksharex-dark-mode') === null) {
      isDark.value = e.matches
      applyDarkMode()
    }
  })
}

export function useDarkMode() {
  return {
    isDark,
    toggleDarkMode,
    initDarkMode,
  }
}
