<script setup lang="ts">
import NavMain from "@/components/NavMain.vue";

import NavSecondary from "@/components/NavSecondary.vue";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  type SidebarProps,
} from "@/components/ui/sidebar";
import { BookOpen, LifeBuoy, Network, Send, Settings } from "lucide-vue-next";

const props = withDefaults(defineProps<SidebarProps>(), {
  variant: "inset",
});

const data = {
  navMain: [
    {
      title: "Network",
      url: "#",
      icon: Network,
      isActive: true,
      items: [
        {
          title: "IPCC",
          url: "/ipcc",
        },
        {
          title: "APN",
          url: "/apn",
        },
      ],
    },
  ],
  navSecondary: [
    {
      title: "Settings",
      url: "/settings",
      icon: Settings,
    },
    {
      title: "Documentation",
      url: "/documentation",
      icon: BookOpen,
    },
    {
      title: "Support",
      url: "/support",
      icon: LifeBuoy,
    },
    {
      title: "Feedback",
      url: "/feedback",
      icon: Send,
    },
  ],
};
</script>

<template>
  <Sidebar v-bind="props">
    <SidebarHeader>
      <Label
        className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
      >
        <BookOpen />
        OpenTools
      </Label>
    </SidebarHeader>
    <SidebarContent>
      <NavMain :items="data.navMain" />
    </SidebarContent>
    <SidebarFooter>
      <NavSecondary :items="data.navSecondary" class="mt-auto" />
    </SidebarFooter>
  </Sidebar>
</template>
