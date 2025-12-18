<script lang="ts">
	import type { FSTree } from '$lib/components/file-explorer';
	import { FileExplorer } from '$lib/components/file-explorer';
	import * as Resizable from '$lib/components/ui/resizable/index.js';

	import * as TreeView from '$lib/components/tree-view';
	import { listen } from '@tauri-apps/api/event';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { getDeviceContext } from '$lib/device-context';

	// getCurrentWebview().onDragDropEvent((event) => {
	// 	if (event.payload.type === 'drop') {
	// 		console.log('User dropped', event.payload.paths);
	// 	} else if (event.payload.type === 'enter') {
	// 		console.log('entered');
	// 	} else if (event.payload.type === 'leave') {
	// 	} else {
	// 		console.log('File drop cancelled');
	// 	}
	// });

	let fsTree: FSTree | null = $state(null);
	let loading = $state<boolean>(false);

	let { connected } = getDeviceContext();

	$effect(() => {
		if ($connected && fsTree === null && !loading) {
			loading = true;
			(async () => {
				try {
					fsTree = await invoke<FSTree>('dump_fs_tree');
					console.log(fsTree);
				} finally {
					loading = false;
				}
			})();
		}
	});
</script>

{#if !$connected}
	Device not connected
{:else if fsTree === null}
	loading ...
{:else}
	<Resizable.PaneGroup direction="horizontal">
		<Resizable.Pane>
			<FileExplorer {fsTree}></FileExplorer>
		</Resizable.Pane>

		<Resizable.Handle withHandle />

		<Resizable.Pane defaultSize={25}>Preview File</Resizable.Pane>
	</Resizable.PaneGroup>
{/if}
