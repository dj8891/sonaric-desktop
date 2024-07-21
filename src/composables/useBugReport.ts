const show = ref(false);
export function useBugReport() {

  return {
    show,
    toggle: () => show.value = !show.value,
  }
}