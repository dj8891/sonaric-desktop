<script setup lang="ts">
const { show } = useBugReport();

</script>

<template>
  <LayoutsDefault>
    <RouterView />
  </LayoutsDefault>

  <transition enter-active-class="transition duration-200 ease-out" enter-from-class="translate-y-1 opacity-0"
    enter-to-class="translate-y-0 opacity-100" leave-active-class="transition duration-150 ease-in"
    leave-from-class="translate-y-0 opacity-100" leave-to-class="translate-y-1 opacity-0">
    <ReportBug v-if="show" />
  </transition>

  <notifications position="top center" />
</template>

<style scoped lang="scss">
html {
  padding: 0;
  margin: 0;
  box-sizing: border-box;
}
</style>
