<script setup lang="ts">
//import { ref } from "vue";
//import { invoke } from "@tauri-apps/api/core";

//const greetMsg = ref("");
//const name = ref("");
//
//async function greet() {
//  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//  greetMsg.value = await invoke("greet", { name: name.value });
//}
import { useColorMode } from "@vueuse/core";
import { RouterView } from "vue-router";
import AppSidebar from "@/components/AppSidebar.vue";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";

useColorMode().value = "dark";
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <SidebarInset>
      <RouterView />
    </SidebarInset>
  </SidebarProvider>
</template>
