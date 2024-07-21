<template>
    <nav class="flex bg-primary-900 p-8 items-center justify-between sticky top-0 border-b-[0.5px] border-b-gray-800">
        <p class="absolute top-0 left-0 text-center w-full bg-primary-400 p-[1px] text-black " v-if="updateAvailable">
            New version
            available, please restart the app to update!
        </p>

        <div class="flex items-center">
            <img class="w-16 h-16 block" src="../../assets/sonaric.png" alt="Sonaric Tracker" />
            <h1 class="text-white text-lg md:text-2xl font-bold pl-2 block ml-2"><span
                    class="text-primary">Sonaric</span> <br>Network</h1>
        </div>
        <ul class="text-white flex list-none">
            <li class="p-2">
                <button href="https://sonaric.xyz" @click="openURL('https://sonaric.xyz')">Home</button>
            </li>
            <li class="p-2">
                <button @click="openURL('https://tracker.sonaric.xyz')">Tracker</button>
            </li>
            <li class="p-2">
                <button :class="{
                    'text-primary': show
                }" @click="toggle">Report a bug</button>
            </li>
        </ul>

        <!-- <div class="flex items-center gap-4">
            <UiButton outlined class="p-2 !rounded-full px-4">Connect Wallet</UiButton>
            <UiButton outlined class="p-2 !rounded-full px-4">Cloud Instance</UiButton>
        </div> -->

    </nav>
</template>
<script lang="ts" setup>
const { toggle, show } = useBugReport();
const { invoke, open } = useTauri()
const version: Ref<{
    app?: {
        up_to_date: boolean
    },
    daemon?: {
        up_to_date: boolean
    }

}> = ref({});

const openURL = async (url: string) => {
    // @ts-ignore
    open(url)
}

const getVersion = async () => {
    if (!window.__TAURI_IPC__) return
    version.value = await invoke('show_version') || {}

}

const updateAvailable = computed(() => version.value?.app?.up_to_date === false || version.value?.daemon?.up_to_date === false)

onMounted(getVersion)
</script>
