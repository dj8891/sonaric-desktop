<template>
  <button id="btn"
    class="sm:font-semibold text-sm sm:text-md text-white rounded-md relative cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
    :class="type + ` ${classes}`" v-bind="$attrs">
    <div class="flex items-center justify-center">
      <UiSpinner v-if="loading" class="w-4 h-4" />
      <slot v-else />
    </div>
  </button>
</template>

<script setup lang="ts">
const props = defineProps({
  type: {
    type: String as PropType<
      "primary" | "error" | "info" | "warning" | "success"
    >,
    default: "primary"
  },
  outlined: {
    type: Boolean,
    default: false
  },
  loading: {
    type: Boolean,
    default: false
  }
})

const classes = computed(() => props.outlined ? `border border-${props.type}  !bg-transparent` : `bg-${props.type}`)
</script>

<style lang="scss" scoped>
.primary {
  @apply bg-primary;
}

.error {
  @apply bg-red-400 text-white;
}

.success {
  @apply bg-green-400 text-white;
}

.info {
  @apply bg-blue-400 text-white;
}

.warning {
  @apply bg-yellow-400 text-white;
}
</style>