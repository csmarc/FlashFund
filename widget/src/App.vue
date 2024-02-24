<script setup lang="ts">
import { invoke } from '@tauri-apps/api/tauri';
import { ref } from 'vue';
const secret = ref('');

async function fetch_secret() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  secret.value = await invoke('start_signer', {});
}

async function fetch_auth_code() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  let message = await invoke('get_auth_message', {});
  await navigator.clipboard.writeText(message as string);
}

fetch_secret();
</script>

<template>
  <div class="container">
    <div class="header">
      <h1>FlashFund ⚡️</h1>
      <a class="withdraw">withdraw</a>
    </div>
    <p v-if="secret">Accepting donations ☑️</p>
    <p v-else>Loading...</p>
    <button @click="fetch_auth_code">Get Login code</button>
  </div>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
