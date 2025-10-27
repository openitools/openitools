<script lang="ts" module>
	import BookOpen from '@lucide/svelte/icons/book-open';
	import LifeBuoy from '@lucide/svelte/icons/life-buoy';
	import Send from '@lucide/svelte/icons/send';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import Network from '@lucide/svelte/icons/network';
	import Play from '@lucide/svelte/icons/play';

	const data = {
		navMain: [
			{
				title: 'Network',
				url: '/network',
				icon: Network,
				isActive: true,
				items: [
					{
						title: 'IPCC',
						url: '/network/ipcc'
					}
					// {
					// 	title: 'APN',
					// 	url: '/network/apn'
					// }
				]
			},
			{
				title: 'Multimedia',
				url: '/multimedia',
				icon: Play,
				isActive: true,
				items: [
					{
						title: 'Screen Mirroring',
						url: '/multimedia/screen-mirroring'
					}
				]
			}
		],
		navSecondary: [
			{
				title: 'Settings',
				url: '/settings',
				icon: Settings2
			},
			{
				title: 'Documentation',
				url: '/docs',
				icon: BookOpen
			},
			{
				title: 'Support',
				url: '/support',
				icon: LifeBuoy
			},
			{
				title: 'Feedback',
				url: '/feedback',
				icon: Send
			}
		]
	};
</script>

<script lang="ts">
	import NavMain from '$lib/components/nav-main.svelte';
	import NavSecondary from '$lib/components/nav-secondary.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import type { ComponentProps } from 'svelte';

	let { ref = $bindable(null), ...restProps }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root bind:ref variant="inset" collapsible="icon" {...restProps}>
	<!-- FIXME: The sidebar header sticks out if closed, that's not good 
        Also, it would be nice if we can put the app icon next to it
  -->
	<Sidebar.Header>OpeniTools</Sidebar.Header>
	<Sidebar.Content>
		<NavMain items={data.navMain} />
	</Sidebar.Content>
	<Sidebar.Footer>
		<NavSecondary items={data.navSecondary} class="mt-auto" />
	</Sidebar.Footer>
</Sidebar.Root>
