<template>
  <div class="wrapper" :class="wrapperClass">
    <label v-if="label" :for="attrs.name || ''" class="block text-base font-medium text-primary">{{ label
      }}</label>
    <div class="mt-[5px]">
      <input v-bind="$attrs"
        class="block w-full rounded-md border-0 text-white shadow-sm ring-1 ring-inset placeholder:text-gray-600  sm:text-sm sm:leading-6 bg-primary-500 p-3 outline-none focus:shadow-input ring-transparent"
        :class="{ 'ring-red-500 focus:ring-red-500 focus:shadow-red-400': hasError || errorMsg }" :value="modelValue"
        @input="(evt: any) => $emit('update:modelValue', evt.target.value)" />
      <p v-if="errorMsg" class="mt-1 text-sm text-red-500">{{ errorMsg }}</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { InputHTMLAttributes } from "vue"

const attrs: InputHTMLAttributes = useAttrs()

defineProps({
  modelValue: {
    type: String,
    default: ""
  },
  wrapperClass: {
    type: String,
    default: ""
  },
  label: {
    type: String,
    default: ""
  },
  hasError: {
    type: Boolean,
    default: false
  },
  errorMsg: {
    type: String,
    default: ""
  }
})
</script>

<style scoped></style>
