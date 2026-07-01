import { ref } from 'vue'

const heroVisible = ref(true)

export function useHeroVisibility() {
  return heroVisible
}
