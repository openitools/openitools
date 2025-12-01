<script lang="ts">
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import FileIcon from '@lucide/svelte/icons/file';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import FileContext from './file-context.svelte';
	import { FileType, type FSTree } from './models';

	import { draggable, droppable, type DragDropState } from '@thisux/sveltednd';
	import { startDrag } from '@crabnebula/tauri-plugin-drag';
	import { invoke } from '@tauri-apps/api/core';

	let selected = $state<string[]>([]);

	function flatten(node: FSTree): string[] {
		return [node.path, ...(node.children?.flatMap(flatten) ?? [])];
	}

	function handleClick(node: FSTree, evt: MouseEvent) {
		const ctrl = evt.ctrlKey || evt.metaKey;
		const shift = evt.shiftKey;

		// so it won't trigger the collapsible
		if (evt.ctrlKey || evt.metaKey || evt.shiftKey) {
			evt.stopPropagation();
			evt.preventDefault();
		}

		if (ctrl) {
			if (selected.some((path) => path === node.path)) {
				selected = selected.filter((path) => path !== node.path);
			} else {
				selected.push(node.path);
			}
		} else if (shift) {
			// TODO: do range select
			selected.push(node.path);
		} else {
			selected = [node.path];
		}
	}

	// global key listener for Ctrl+A
	window.addEventListener('keydown', (e) => {
		if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') {
			e.preventDefault();
			selected = flatten(fsTree);
		}
	});

	function handleInnerDrop(state: DragDropState<FSTree>) {
		const { draggedItem, targetContainer } = state;
		if (!targetContainer) {
			console.log('nothing');
			return;
		}
		console.log('Dropped', draggedItem, 'onto', targetContainer);
	}

	async function handleDrag(node: FSTree) {
		let filesToCopy: string[] = [];

		if (selected.includes(node.path)) {
			filesToCopy = selected;
		} else {
			filesToCopy = [...selected, node.path];
		}

		console.log(filesToCopy);

		const tempFilesPath: string[] = await invoke<string[]>('mount_fuse', {
			filesPath: filesToCopy
		});

		// const tempFilePaths: string[] = ['/tmp/.tar/zen-x86_64.AppImage'];
		console.log(tempFilesPath);

		await startDrag({ item: tempFilesPath, icon: '/home/abdullah/file.patch' });
	}

	const isFile = (node: FSTree) => node.info.file_type.toString() === 'File';

	let { fsTree }: { fsTree: FSTree } = $props();
</script>

{#each fsTree.children as node (node.path)}
	{@render Tree({ node })}
{/each}

{#snippet Tree({ node }: { node: FSTree })}
	<!-- <div use:droppable={{ container: node.path, callbacks: { onDrop: handleInnerDrop } }}> -->
	<!-- 	<div -->
	<!-- 		use:draggable={{ -->
	<!-- 			container: node.path, -->
	<!-- 			dragData: node -->
	<!-- 		}} -->
	<!-- 	> -->
	{#if isFile(node)}
		<Sidebar.MenuButton
			isActive={selected.includes(node.path)}
			class={`data-[active=${!selected.includes(node.path)}]:bg-transparent`}
			onclick={(e) => handleClick(node, e)}
			draggable={true}
			ondragstart={(e) => {
				e.preventDefault();

				handleDrag(node);
			}}
		>
			<!--FIXME: so it aligns with the folders, probably not a good idea-->
			<ChevronRightIcon class="invisible" />
			<FileIcon />

			{node.path}
		</Sidebar.MenuButton>
	{:else}
		<Sidebar.MenuItem
			draggable={true}
			ondragstart={(e) => {
				e.preventDefault();
				handleDrag(node);
			}}
		>
			<Collapsible.Root
				class="group/collapsible [&[data-state=open]>button>button>svg:first-child]:rotate-90"
			>
				<Collapsible.Trigger class="w-full">
					<Sidebar.MenuButton
						isActive={selected.includes(node.path)}
						class={`data-[active=${!selected.includes(node.path)}]`}
						onclick={(e) => handleClick(node, e)}
					>
						<ChevronRightIcon class="transition-transform " />
						<FolderIcon />
						{node.path}
					</Sidebar.MenuButton>
				</Collapsible.Trigger>

				<Collapsible.Content>
					<Sidebar.MenuSub>
						{#each node.children as sub (sub.path)}
							{@render Tree({ node: sub })}
						{/each}
					</Sidebar.MenuSub>
				</Collapsible.Content>
			</Collapsible.Root>
		</Sidebar.MenuItem>
	{/if}
	<!-- </div> -->
	<!-- </div> -->
{/snippet}
