<script lang="ts">
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';

	import * as Accordion from '$lib/components/ui/accordion/index.js';
	import * as Drawer from '$lib/components/ui/drawer/index.js';

	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import { onMount, tick } from 'svelte';
	import * as Command from '$lib/components/ui/command/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { cn, when } from '$lib/utils.js';
	import { invoke } from '@tauri-apps/api/core';
	import { getDeviceContext, empty_hardware, empty_os } from '$lib/device-context';
	import { listen } from '@tauri-apps/api/event';

	let open = $state(false);
	let value = $state('');
	let triggerRef = $state<HTMLButtonElement>(null!);
	let loading = $state<boolean>(true);

	let installResult = $state<'idle' | 'success' | 'error'>('idle');

	let bundles = $state<{ bundle_name: string }[]>([]);
	const { hardware, os, connected } = getDeviceContext();

	let openItem: string | null = 'linux';
	$effect(() => {
		(async () => {
			if ($connected) {
				const hw = $hardware;
				const _os = $os;

				console.log(hw);
				console.log(_os);

				if (hw == empty_hardware || _os == empty_os) {
					return;
				}

				const result = (await invoke('get_bundles', {
					deviceModel: $hardware.model,
					iosVersion: $os.ios_ver
				})) as {
					bundle_name: string;
				}[];

				bundles = result;
				loading = false;
			} else {
				console.log('Not Connected');
			}
		})();
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
				expectedModel: $hardware.model,
				expectedIosVersion: $os.ios_ver,
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
	<Card.Root class="flex h-screen flex-col items-center justify-center gap-4 p-4">
		<Card.Header class="text-center">
			<Card.Title class="text-2xl font-semibold">Device Not Connected</Card.Title>
			<Card.Description class="text-muted-foreground">
				Please connect your device to continue.
			</Card.Description>
		</Card.Header>
		<Card.Content>
			<Drawer.Root>
				<Drawer.Trigger class={buttonVariants({ variant: 'outline' })}>Need Help?</Drawer.Trigger>

				<Drawer.Content>
					<div class="mx-auto w-full max-w-md">
						<Drawer.Header>
							<Drawer.Title>Connection Troubleshooting</Drawer.Title>
							<Drawer.Description>
								Fix common issues based on your operating system.
							</Drawer.Description>
						</Drawer.Header>

						<div class="p-4">
							<Accordion.Root type="single" class="w-full" value="linux">
								<Accordion.Item value="linux">
									<Accordion.Trigger>Linux</Accordion.Trigger>
									<Accordion.Content class="prose flex flex-col gap-4 text-balance">
										<p>
											Make sure <strong>usbmuxd</strong> is installed and running.
										</p>
										<ul class="ml-6 list-disc">
											<li>
												Install usbmuxd:
												<pre class="bg-muted rounded px-3 py-2"><code
														>sudo pacman -S usbmuxd
sudo systemctl start usbmuxd
sudo systemctl enable usbmuxd</code
													></pre>
											</li>
											<li>
												Verify the service is running:
												<pre class="bg-muted rounded px-3 py-2"><code>systemctl status usbmuxd</code
													></pre>
											</li>
										</ul>
									</Accordion.Content>
								</Accordion.Item>

								<Accordion.Item value="windows">
									<Accordion.Trigger>Windows</Accordion.Trigger>
									<Accordion.Content class="prose flex flex-col gap-4 text-balance">
										<p>
											Ensure <strong>iTunes</strong> is installed and
											<strong>Apple Mobile Device Service</strong> is running.
										</p>
										<ul class="ml-6 list-disc">
											<li>Press <code>Win + R</code> → type <code>services.msc</code></li>
											<li>Find <strong>Apple Mobile Device Service</strong> → Start/Restart</li>
										</ul>
									</Accordion.Content>
								</Accordion.Item>

								<Accordion.Item value="macos">
									<Accordion.Trigger>macOS</Accordion.Trigger>
									<Accordion.Content class="prose flex flex-col gap-4 text-balance">
										<p>Ensure your device is trusted and drivers are up to date.</p>
									</Accordion.Content>
								</Accordion.Item>
							</Accordion.Root>
						</div>

						<Drawer.Footer>
							<Drawer.Close class={buttonVariants({ variant: 'outline' })}>Close</Drawer.Close>
						</Drawer.Footer>
					</div>
				</Drawer.Content>
			</Drawer.Root>
		</Card.Content>
	</Card.Root>
{:else if loading}
	<div class="flex h-screen w-full items-center justify-center">
		<div class="border-muted border-t-primary h-10 w-10 animate-spin rounded-full border-4"></div>
	</div>
{:else}
	<Card.Root class="flex h-full w-full flex-1 flex-col items-center justify-center p-4">
		<Card.Header class="space-y-1 text-center">
			<Card.Title class="text-foreground text-2xl font-semibold">Install IPCC</Card.Title>
			<Card.Description class="text-muted-foreground text-sm">
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
