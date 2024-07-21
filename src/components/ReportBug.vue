<template>
  <div class="p-4 fixed border border-gray-600 right-8 bottom-8 rounded-lg mt-4 bg-primary-600 z-20 w-[400px]">

    <h4 class="text-xl mb-4">Report a Bug</h4>
    <UiInput label="Name" placeholder="Your Name" v-model="report.name" />
    <p class="mt-4 mb-1 text-primary ">Description</p>
    <textarea class="bg-primary-500 rounded-lg p-4 text-sm w-full outline-none placeholder:text-gray-600"
      v-model="report.description" placeholder="What's the bug? What did you expect?" rows="8"></textarea>

    <UiButton class="p-3 w-full mt-4 !text-black" :disabled="disabled" :loading="loading" @click="submitReport">Send Bug
      Report
    </UiButton>
    <UiButton class="!border-gray-500 p-3 w-full mt-4" outlined @click.native="toggle">Cancel</UiButton>
  </div>
</template>

<script setup lang="ts">
import { useNotification } from '@kyvg/vue3-notification';
const { toggle } = useBugReport();
const { notify } = useNotification();
const { invoke } = useTauri();
const report = reactive({
  name: '',
  description: ''
})
const loading = ref(false);
const disabled = computed(() => !report.description?.trim()?.length
  || !report.name?.trim()?.length)
const submitReport = async () => {
  try {
    loading.value = true
    await invoke('report_bug', report)
    notify({
      text: 'Bug report sent successfully',
      type: 'success'
    })
    toggle()
  } catch (error) {
    notify({
      text: 'Failed to send bug report',
      type: 'error'
    })
    console.error(error)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped></style>