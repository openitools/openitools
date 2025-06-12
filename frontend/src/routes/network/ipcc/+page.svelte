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

	const { hardware, os, connected } = getDeviceContext();

	let open = $state(false);
	let value = $state('');
	let triggerRef = $state<HTMLButtonElement>(null!);
	let loading = $state<boolean>(true);

	let frameworks = $state<{ bundle_name: string }[]>([]);

	onMount(async () => {
		try {
			await when(connected);
			const result = (await invoke('get_bundles', {
				model: $hardware.model,
				version: $os.ios_ver
			})) as {
				bundle_name: string;
			}[];
			frameworks = result;
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
</script>

{#if !$connected}
	connect the device
{:else if loading}
	<!-- Skeleton or loading UI -->
	<div class="flex h-screen w-full items-center justify-center">
		<div class="h-10 w-10 animate-spin rounded-full border-4 border-muted border-t-primary"></div>
	</div>
{:else}
	<Card.Root class="flex h-full w-full flex-1 flex-col items-center justify-center p-4">
		<Card.Header class="space-y-1 text-center">
			<Card.Title class="text-2xl font-semibold text-foreground">Inject IPCC</Card.Title>
			<Card.Description class="text-sm text-muted-foreground">
				Select the bundle and hit inject
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
							<Command.Empty>No framework found.</Command.Empty>
							<Command.Group value="frameworks">
								{#each frameworks as framework}
									<Command.Item
										value={framework.bundle_name}
										onSelect={() => {
											value = framework.bundle_name;
											closeAndFocusTrigger();
										}}
									>
										<CheckIcon class={cn(value !== framework.bundle_name && 'text-transparent')} />
										{framework.bundle_name}
									</Command.Item>
								{/each}
							</Command.Group>
						</Command.List>
					</Command.Root>
				</Popover.Content>
			</Popover.Root>
		</Card.Content>

		<Card.Footer class="flex w-full flex-col gap-2">
			<Button class="w-full" onclick={() => {}}>Inject</Button>
		</Card.Footer>
	</Card.Root>
{/if}
