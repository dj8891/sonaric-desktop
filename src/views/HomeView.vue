<template>
  <div class="p-12">
    <div class="" v-if="installing">
      <h1 class="text-xl text-center">{{ actionText }}</h1>
      <div
        class="logs text-sm bg-primary-600 rounded p-4 mt-4 h-[calc(100vh-230px)] max-h-[calc(100vh-380px)] overflow-y-auto w-full overflow-x-hidden flex flex-col-reverse"
        v-if="logs.length">
        <p v-for="(log, i) in logs" :key="i" class="text-sm">{{ log }}</p>
      </div>

      <div class="progress-bar mt-4">
        <div class="progress-bar-inner"></div>
      </div>
    </div>
    <template v-else>
      <div class="w-[700px] flex flex-col items-center mx-auto" v-if="isEula">
        <textarea class="w-full bg-transparent resize-none mb-3 h-[calc(100vh-325px)] focus:outline-none" readonly>
          Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
        </textarea>
        <UiToggle label="I accept the terms in the License Agreement" v-model="isChecked" />
        <UiButton :disabled="!isChecked" @click="toggle" class="px-8 py-2 !text-black block mt-4 !rounded-full mx-auto !bg-primary border-none">OK</UiButton>
      </div>
      <template v-else>
        <h1 class="text-xl text-center">Welcome to Sonaric!</h1>
        <div class="flex items-center justify-center mt-4" v-if="isLoading" ><UiSpinner class="w-8 h-8"/></div>
        <UiButton v-else class="px-4 py-2 !text-black block mt-4 !rounded-full mx-auto !bg-primary border-none" id="install-btn" @click="installDeps">
          {{ buttonLabel }}
        </UiButton>

        <p class="mt-4 text-gray-400 text-center" id="install-process">{{ installProcText }}</p>
        <p id="greet-msg" class="bg-primary-600 mt-4 rounded-lg text-center whitespace-pre-line">{{ greetMsgText }}</p>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import {appWindow} from '@tauri-apps/api/window'
import ms from 'ms';

const {invoke, listen} = useTauri()
const actionText = ref('')
const installProcText = ref('')
const greetMsgText = ref('')
const installing = ref(false)
const buttonLabel = ref('Install Sonaric')
const logs: Ref<String[]> = ref([])
const isLoading = ref(false);
const isEula = ref(false);
const isChecked = ref(false);

async function checkInstall() {
  greetMsgText.value = ''
  await invoke('check_install').then(async (msg) => {
    isLoading.value = false;
    console.log(msg)
    if (msg === 'OK') {
      try {
        await appWindow.maximize()
      } catch (error) {
        console.log('Error maximizing window: ', error)
      }
      window.location.href = 'http://localhost:44004'
    } else {
      switch (msg) {
        case 'install':
          isEula.value = true;
          buttonLabel.value = 'Install Sonaric Node'
          installProcText.value = 'Sonaric is not installed. Click Install to proceed.'
          actionText.value = 'Sonaric is being installed...';
          break
        case 'start':
          buttonLabel.value = 'Start Sonaric Node'
          installProcText.value = 'Sonaric is not running. Click Start to proceed.'
          actionText.value = 'Starting Sonaric';
          break
        case 'update':
          buttonLabel.value = 'Update Sonaric Node'
          installProcText.value = 'There is an update available. Click Update to proceed.'
          actionText.value = 'Updating Sonaric';
          break
        default:
          installProcText.value = 'Click Install to proceed.'
          actionText.value = 'Installing Sonaric';
          break
      }
    }
  })
}

async function checkGUI() {
  greetMsgText.value = ''
  await invoke('check_gui').then(async (msg) => {
    console.log(msg)
    if (msg === 'OK') {
      try {
        await appWindow.maximize()
      } catch (error) {
        console.log('Error maximizing window: ', error)
      }
      window.location.href = 'http://localhost:44004'
    }
  })
}

async function installDeps() {
  try {
    installing.value = true
    const time = Date.now();
    logs.value.unshift('Check dependencies...')
    greetMsgText.value = ''
    greetMsgText.value = await invoke('install_deps')
    // installProcText.value = 'Install done'
    logs.value.unshift('Finished dependencies check in ' + ms(Date.now() - time))

    // check with retry because the start process may take some time
    const id = setInterval(function () {
      checkGUI().finally(() => {
        // installProcText.value = 'Waiting for Sonaric to become ready...'
        if (logs.value[0] !== 'Waiting for Sonaric to become ready...') logs.value.unshift('Waiting for Sonaric to become ready...')
      })
    }, 2000)

    // stop retry after 60 seconds
    setTimeout(() => {
      clearInterval(id)

      // check one more time and show error if not ready
      checkGUI()
        .then(() => {
          // installProcText.value = 'Waiting for Sonaric to become ready...'
          logs.value.unshift('Waiting for Sonaric to become ready...')
        })
        .catch((err) => {
          installing.value = false
          installProcText.value = 'Install error'
          greetMsgText.value = 'Install error: ' + err
          console.error(err)
        })
    }, 300000)
  } catch (error) {
    isLoading.value = false;
    installProcText.value = 'Error'
    installing.value = false
    greetMsgText.value = 'Error: ' + error
    console.log(error)
  }
}

async function doAction(action: string, msg: string) {
  try {
    installing.value = true
    actionText.value = msg
    const time = Date.now();
    logs.value.unshift(msg + '...')
    greetMsgText.value = ''
    greetMsgText.value = await invoke(action)
    // installProcText.value = 'Install done'
    logs.value.unshift(msg + ' finished in ' + ms(Date.now() - time))
  } catch (error) {
    installProcText.value = 'Error'
    installing.value = false
    greetMsgText.value = 'Error: ' + error
    console.log(error)
  }

  checkInstall().catch(() => {
    isLoading.value = false;
    installProcText.value = 'Please click Install to proceed'
    greetMsgText.value = ''
  });
}

const toggle = () => {
  isEula.value = !isEula.value;
}

onMounted(() => {
  listen('status', (msg) => {
    console.log('status: ', msg)
    isLoading.value = true;
    installProcText.value = String(msg.payload);
  })

  listen('install-output', (msg) => {
    console.log('install-output: ', msg)
    greetMsgText.value = String(msg.payload)
    logs.value.unshift(String(msg.payload))
  })

  let params = new URL(window.location.toString()).searchParams;
  switch (params.get("action")) {
    case "uninstall":
      doAction('uninstall_daemon', 'Uninstalling Sonaric').then(() => {
        window.location.href = '/'
      })
      break;
    case "stop":
      doAction('stop_daemon', 'Stopping Sonaric').then(() => {
        window.location.href = '/'
      })
      break;
    default:
      checkInstall().catch(() => {
        isLoading.value = false;
        installProcText.value = 'Please click Install to proceed'
        greetMsgText.value = ''
      });
      break
  }
})
</script>

<style scoped lang="scss">
.logs {
  &::-webkit-scrollbar {
    width: 2px;
  }

  &::-webkit-scrollbar-thumb {
    @apply bg-primary;
    border-radius: 4px;
  }

  &::-webkit-scrollbar-track {
    @apply bg-gray-800;
  }
}

.progress-bar {
  width: 100%;
  height: 6px;
  background-color: #ddd;
  overflow: hidden;

  .progress-bar-inner {
    height: 100%;
    background: linear-gradient(90deg, #22d3ee88, #0d9488);
    animation: progress-bar-animation 2s linear infinite;
  }
}

@keyframes progress-bar-animation {
  0% {
    transform: translateX(-100%);
  }

  100% {
    transform: translateX(100%);
  }
}
</style>
