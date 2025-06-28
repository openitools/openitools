<script lang="ts">
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';

	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import { onMount, tick } from 'svelte';
	import * as Command from '$lib/components/ui/command/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { cn, when } from '$lib/utils.js';
	import { invoke } from '@tauri-apps/api/core';
	import { getDeviceContext } from '$lib/device-context';
	import { listen } from '@tauri-apps/api/event';

	const { hardware, os, connected } = getDeviceContext();

	let open = $state(false);
	let value = $state('');
	let triggerRef = $state<HTMLButtonElement>(null!);
	let loading = $state<boolean>(true);

	let installResult = $state<'idle' | 'success' | 'error'>('idle');

	let bundles = $state<{ bundle_name: string }[]>([]);

	onMount(async () => {
		try {
			await when(connected);
			const result = (await invoke('get_bundles', {
				deviceModel: $hardware.model,
				iosVersion: $os.ios_ver
			})) as {
				bundle_name: string;
			}[];
			bundles = result;
			loading = false;
		} catch (e) {
			console.error('Failed to load frameworks from Rust:', e);
		}
	});

	// We want to refocus the trigger button when the user selects
	// an item from the list so users can continue navigating the
	// rest of the form with the keyboard.
	function closeAndFocusTrigger() {
		open = false;
		tick().then(() => {
			triggerRef.focus();
		});
	}

	async function install() {
		installResult = 'idle';

		try {
			const waitForEmit = new Promise<boolean>((resolve, reject) => {
				listen<boolean>('carrier_bundle_install_status', (event) => {
					console.log('Received event:', event);
					resolve(event.payload);
					unlisten();
				})
					.then((unsub) => {
						unlisten = unsub;
					})
					.catch(reject);

				let unlisten: () => void;

				setTimeout(() => {
					if (unlisten) {
						unlisten();
						reject(new Error('Timed out waiting for carrier_bundle_install_status'));
					}
				}, 10_000); // 10s timeout
			});

			await invoke('install_ipcc', {
				deviceModel: $hardware.model,
				iosVersion: $os.ios_ver,
				bundle: value
			});

			const result = await waitForEmit;

			installResult = result ? 'success' : 'error';

			await new Promise((r) => setTimeout(r, 3000));
			installResult = 'idle';
		} catch (e) {
			console.error('Install failed:', e);
			installResult = 'error';

			await new Promise((r) => setTimeout(r, 3000));
			installResult = 'idle';
		}
	}
</script>

{#if !$connected}
	connect the device
{:else if loading}
	<div class="flex h-screen w-full items-center justify-center">
		<div class="h-10 w-10 animate-spin rounded-full border-4 border-muted border-t-primary"></div>
	</div>
{:else}
	<Card.Root class="flex h-full w-full flex-1 flex-col items-center justify-center p-4">
		<Card.Header class="space-y-1 text-center">
			<Card.Title class="text-2xl font-semibold text-foreground">Install IPCC</Card.Title>
			<Card.Description class="text-sm text-muted-foreground">
				Select the bundle and hit install
			</Card.Description>
		</Card.Header>

		<Card.Content class="flex w-full flex-1 flex-col items-center justify-center gap-4">
			<Popover.Root bind:open>
				<Popover.Trigger bind:ref={triggerRef}>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="outline"
							class="w-[200px] justify-between"
							role="combobox"
							aria-expanded={open}
						>
							{value || 'Select a bundle...'}
							<ChevronsUpDownIcon class="opacity-50" />
						</Button>
					{/snippet}
				</Popover.Trigger>
				<Popover.Content class="w-[200px] p-0">
					<Command.Root>
						<Command.Input placeholder="Search a bundle.." />
						<Command.List>
							<Command.Empty>No bundle found.</Command.Empty>
							<Command.Group value="frameworks">
								{#each bundles as bundle}
									<Command.Item
										value={bundle.bundle_name}
										onSelect={() => {
											value = bundle.bundle_name;
											closeAndFocusTrigger();
										}}
									>
										<CheckIcon class={cn(value !== bundle.bundle_name && 'text-transparent')} />
										{bundle.bundle_name}
									</Command.Item>
								{/each}
							</Command.Group>
						</Command.List>
					</Command.Root>
				</Popover.Content>
			</Popover.Root>
		</Card.Content>

		<Card.Footer class="flex w-full flex-col gap-2">
			<Button
				class={cn(
					'w-full',
					installResult === 'success' && 'bg-green-500 text-white hover:bg-green-600',
					installResult === 'error' && 'bg-red-500 text-white hover:bg-red-600'
				)}
				onclick={install}
			>
				{#if installResult === 'success'}
					Installed
				{:else if installResult === 'error'}
					Failed
				{:else}
					Install
				{/if}
			</Button>
		</Card.Footer>
	</Card.Root>
{/if}
