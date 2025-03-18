<script lang="ts">
	import '../app.css';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import SidebarProvider from '$lib/components/ui/sidebar/sidebar-provider.svelte';

	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { afterNavigate } from '$app/navigation';

	import * as ModeWatcher from 'mode-watcher';
	ModeWatcher.setMode('system');

	let currentRoute = $state<string[]>([]);
	afterNavigate((navigation) => {
		currentRoute = navigation.to?.url.pathname.split('/') as string[];
	});

	let { children } = $props();

	/*

				<Separator orientation="vertical" class="mr-2 h-4" />

						<Breadcrumb.Item class="hidden md:block">
							<Breadcrumb.Link href="#">Building Your Application</Breadcrumb.Link>
						</Breadcrumb.Item>
						<Breadcrumb.Separator class="hidden md:block" />
						<Breadcrumb.Item>
							<Breadcrumb.Page>Data Fetching</Breadcrumb.Page>
						</Breadcrumb.Item>
*/

	function routes_to_display(path: string): string {
		switch (path.toLowerCase()) {
			case 'network':
			case 'settings':
			case 'feedback':
			case 'support':
				return path.charAt(0).toUpperCase() + path.slice(1);

			case 'ipcc':
				return 'IPCC';
			case 'apn':
				return 'APN';

			case 'docs':
				return 'Documentation';

			default:
				return path;
		}
	}
</script>

<SidebarProvider>
	<ModeWatcher.ModeWatcher />
	<AppSidebar />

	<Sidebar.Inset>
		<header class="flex h-16 shrink-0 items-center gap-2">
			<div class="flex items-center gap-2 px-4">
				<Sidebar.Trigger class="-ml-1" />
				<Breadcrumb.Root>
					<Breadcrumb.List>
						{#each currentRoute as path, i (i)}
							<Breadcrumb.Item class="hidden cursor-pointer md:block">
								<Breadcrumb.Link>{routes_to_display(path)}</Breadcrumb.Link>
							</Breadcrumb.Item>
							{#if currentRoute.length !== i + 1}
								<Breadcrumb.Separator class="hidden md:block" />
								<Separator orientation="vertical" class="mr-2 h-4" />
							{/if}
						{/each}
					</Breadcrumb.List>
				</Breadcrumb.Root>
			</div>
		</header>

		{@render children()}
		<div class="flex flex-1 flex-col gap-4 p-4 pt-0">
			<div class="grid auto-rows-min gap-4 md:grid-cols-3">
				<div class="aspect-video rounded-xl bg-muted/50"></div>
				<div class="aspect-video rounded-xl bg-muted/50"></div>
				<div class="aspect-video rounded-xl bg-muted/50"></div>
			</div>
			<div class="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min"></div>
		</div>
	</Sidebar.Inset>
</SidebarProvider>
