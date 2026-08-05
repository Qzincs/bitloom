<script lang="ts">
  import ProtocolTreeItem from "./ProtocolTreeItem.svelte";
  import ProtocolCreateInput from "$lib/components/ProtocolCreateInput.svelte";
  import ProtocolFileIcon from "$lib/components/ui/ProtocolFileIcon.svelte";
  import TreeChevron from "$lib/components/ui/TreeChevron.svelte";
  import type { ProtocolNode } from "./protocol-tree";

  const TREE_INDENT_BASE = 16;
  const TREE_INDENT_STEP = 8;

  interface Props {
    node: ProtocolNode;
    depth: number;
    selectedId: string | null;
    onSelect: (id: string) => void;
    onContextMenu: (event: MouseEvent, id: string) => void;
    renamingId: string | null;
    renameDraft: string;
    onRenameCommit: () => void;
    onRenameCancel: () => void;
    ancestorActive: boolean[];
    isCreating: boolean;
    draftParentId: string | null;
    draftId: string;
    draftError: string;
    onDraftCommit: () => void;
    onDraftCancel: () => void;
  }

  let {
    node,
    depth,
    selectedId,
    onSelect,
    onContextMenu,
    renamingId,
    renameDraft = $bindable(),
    onRenameCommit,
    onRenameCancel,
    ancestorActive,
    isCreating,
    draftParentId,
    draftId = $bindable(),
    draftError,
    onDraftCommit,
    onDraftCancel
  }: Props = $props();

  let hasChildren = $derived((node.children?.length ?? 0) > 0);
  let isExpanded = $state(true);
  let selected = $derived(node.id === selectedId);
  function containsSelected(node: ProtocolNode, selectedId: string | null): boolean {
    return (
      node.id === selectedId ||
      (node.children?.some((child) => containsSelected(child, selectedId)) ?? false)
    );
  }

  let containsSelectedNode = $derived(containsSelected(node, selectedId));
  let indentation = $derived(
    TREE_INDENT_BASE + depth * TREE_INDENT_STEP
  );

  function toggleExpanded(event: MouseEvent) {
    event.stopPropagation();
    isExpanded = !isExpanded;
  }

  function moveVertically(event: KeyboardEvent) {
    if (event.key !== "ArrowUp" && event.key !== "ArrowDown") {
      return;
    }

    event.preventDefault();
    const rows = Array.from(
      document.querySelectorAll<HTMLButtonElement>("[data-protocol-id]")
    );
    const currentIndex = rows.findIndex(
      (row) => row.dataset.protocolId === node.id
    );
    const offset = event.key === "ArrowUp" ? -1 : 1;
    const target = rows[currentIndex + offset];

    if (target?.dataset.protocolId) {
      onSelect(target.dataset.protocolId);
      target.focus();
    }
  }
</script>

{#snippet indentArea(activeGuides: boolean[], width: number)}
  <span
    class="relative flex h-full shrink-0"
    style:width={`${width}px`}
    aria-hidden="true"
  >
    <span class="w-4 shrink-0"></span>
    {#each activeGuides as active}
      <span class="relative h-full w-2 shrink-0">
        <span
          class={`absolute inset-y-0 left-2 w-[0.5px] ${active ? "bg-foreground/50" : "bg-foreground/15 opacity-0 group-hover/tree:opacity-100"}`}
        ></span>
      </span>
    {/each}
  </span>
{/snippet}

{#if renamingId === node.id}
  <div
    class="relative flex h-[22px] items-center gap-1"
    data-protocol-rename
    style={`margin-left: -${indentation}px; width: calc(100% + ${indentation}px);`}
  >
    {@render indentArea(ancestorActive, indentation)}
    {#if hasChildren}
      <span class="flex size-4 shrink-0 items-center" aria-hidden="true">
        <TreeChevron expanded={isExpanded} />
      </span>
    {:else}
      <ProtocolFileIcon />
    {/if}
    <ProtocolCreateInput
      bind:value={renameDraft}
      placeholder="备注"
      ariaLabel="Protocol remark"
      onCommit={onRenameCommit}
      onCancel={onRenameCancel}
    />
  </div>
{:else}
<button
  type="button"
  class={`relative flex h-[22px] cursor-pointer items-center gap-1 text-left outline-none ${selected ? "bg-foreground/[0.12] text-foreground" : "hover:bg-foreground/[0.06]"}`}
  style={`margin-left: -${indentation}px; width: calc(100% + ${indentation}px);`}
  data-protocol-id={node.id}
  aria-current={selected ? "true" : undefined}
  onclick={() => onSelect(node.id)}
  oncontextmenu={(event) => onContextMenu(event, node.id)}
  onkeydown={moveVertically}
>
  {@render indentArea(ancestorActive, indentation)}

  {#if hasChildren}
    <span
      class="flex size-4 shrink-0 cursor-pointer items-center"
      title={isExpanded ? "Collapse" : "Expand"}
      onclick={toggleExpanded}
      aria-hidden="true"
    >
      <TreeChevron expanded={isExpanded} />
    </span>
  {:else}
    <ProtocolFileIcon />
  {/if}
  <span class="flex min-w-0 truncate">
    <span class="truncate">{node.id}</span>
    {#if node.remark && node.remark !== node.id}
      <span class="ml-1 shrink-0 text-muted-foreground">({node.remark})</span>
    {/if}
  </span>
</button>
{/if}

{#if (hasChildren && isExpanded) || (isCreating && draftParentId === node.id)}
  <div class="ml-2">
    {#if isCreating && draftParentId === node.id}
      {@const draftDepth = depth + 1}
      {@const draftIndentation = TREE_INDENT_BASE + draftDepth * TREE_INDENT_STEP}
      <div
        class="relative flex h-[22px] items-center gap-1"
        data-protocol-create
        style={`margin-left: -${draftIndentation}px; width: calc(100% + ${draftIndentation}px);`}
      >
        {@render indentArea([...ancestorActive, containsSelectedNode], draftIndentation)}
        <ProtocolFileIcon />
        <ProtocolCreateInput
          bind:value={draftId}
          onCommit={onDraftCommit}
          onCancel={onDraftCancel}
        />
      </div>
      {#if draftError}
        <p class="px-1 text-[11px] text-destructive">{draftError}</p>
      {/if}
    {/if}

    {#each node.children ?? [] as child (child.id)}
      <ProtocolTreeItem
        node={child}
        depth={depth + 1}
        {selectedId}
        {onSelect}
        {onContextMenu}
        {renamingId}
        bind:renameDraft
        {onRenameCommit}
        {onRenameCancel}
        ancestorActive={[...ancestorActive, containsSelectedNode]}
        {isCreating}
        {draftParentId}
        bind:draftId
        {draftError}
        {onDraftCommit}
        {onDraftCancel}
      />
    {/each}
  </div>
{/if}
