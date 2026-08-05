<script lang="ts">
  import { tick } from "svelte";

  export interface ContextMenuItem {
    id: string;
    label: string;
    disabled?: boolean;
  }

  interface Props {
    open: boolean;
    x: number;
    y: number;
    items: ContextMenuItem[];
    onSelect: (id: string) => void;
    onClose: () => void;
  }

  let { open, x, y, items, onSelect, onClose }: Props = $props();
  let menuElement = $state<HTMLDivElement | null>(null);
  let position = $state({ x: 0, y: 0 });

  async function updatePosition() {
    await tick();
    if (!menuElement) return;

    const rect = menuElement.getBoundingClientRect();
    position = {
      x: Math.max(8, Math.min(x, window.innerWidth - rect.width - 8)),
      y: Math.max(8, Math.min(y, window.innerHeight - rect.height - 8))
    };
  }

  $effect(() => {
    if (open) {
      updatePosition();
    }
  });

  function handleWindowResize() {
    if (open) {
      updatePosition();
    }
  }

  function handleWindowPointerDown(event: PointerEvent) {
    if (!open) return;
    const target = event.target;
    if (target instanceof Element && target.closest("[data-context-menu]")) {
      return;
    }
    onClose();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  }

  function handleWindowBlur() {
    if (open) {
      onClose();
    }
  }
</script>

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onkeydown={handleWindowKeydown}
  onresize={handleWindowResize}
  onblur={handleWindowBlur}
/>

{#if open}
  <div
    class="fixed z-50 min-w-40 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
    bind:this={menuElement}
    style:left={`${position.x}px`}
    style:top={`${position.y}px`}
    data-context-menu
    role="menu"
  >
    {#each items as item (item.id)}
      <button
        type="button"
        class="flex h-7 w-full items-center rounded-sm px-2 text-left text-[13px] outline-none hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50"
        role="menuitem"
        disabled={item.disabled}
        onclick={() => onSelect(item.id)}
      >
        {item.label}
      </button>
    {/each}
  </div>
{/if}
