<script setup lang="ts">
import {
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";

import { defineProps } from "vue";

defineProps<{
  items: string[];
}>();
</script>

<template>
  <BreadcrumbItem v-for="(path, index) in items" :key="path">
    <BreadcrumbLink v-if="index !== items.length - 1" :href="path">
      {{ path }}
    </BreadcrumbLink>
    <BreadcrumbPage v-else>
      {{ path }}
    </BreadcrumbPage>
    <BreadcrumbSeparator v-if="index !== items.length - 1" />
  </BreadcrumbItem>
</template>
