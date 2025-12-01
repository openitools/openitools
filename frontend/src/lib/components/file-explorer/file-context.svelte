<script lang="ts">
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';

	let { children } = $props();
</script>

<ContextMenu.Root>
	<ContextMenu.Trigger>{@render children()}</ContextMenu.Trigger>
	<ContextMenu.Content>
		<ContextMenu.Item>
			Save
			<ContextMenu.Shortcut>^S</ContextMenu.Shortcut>
		</ContextMenu.Item>
	</ContextMenu.Content>
</ContextMenu.Root>
