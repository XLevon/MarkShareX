import { ref } from 'vue'

/**
 * Controls whether the navbar search box should be visible.
 * Set to `false` when the hero search box (on homepage) is in viewport.
 * Default: `true` (visible when not on homepage or hero scrolled away).
 */
export const navSearchVisible = ref(true)
