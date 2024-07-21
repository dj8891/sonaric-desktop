<template>
  <label class="inline-flex items-center cursor-pointer">
    <input type="checkbox" :value="modelValue" class="sr-only peer" @input="$emit('update:modelValue', !modelValue)">
    <div
      class="relative w-11 h-6 bg-gray-200  rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary">
    </div>
    <span class="ms-3 text-sm">
      {{ label }}
    </span>

  </label>
</template>

<script setup lang="ts">
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  label: {
    type: String,
    default: ''
  }
})

defineEmits(['update:modelValue'])
console.log("this is ddd", props.modelValue)
</script>

<style scoped></style>