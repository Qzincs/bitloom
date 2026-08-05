<script lang="ts">
  export interface FieldRow {
    id: string;
    remark: string;
    kind: string;
    length: string;
    offset: string;
  }

  type EditableFieldProperties = Pick<FieldRow, "remark" | "kind" | "length">;

  interface Props {
    rows: FieldRow[];
    selectedId: string | null;
    onSelect: (id: string) => void;
    onEdit: (id: string, patch: Partial<EditableFieldProperties>) => void;
  }

  let { rows, selectedId, onSelect, onEdit }: Props = $props();

  const gridColumns =
    "grid-cols-[minmax(90px,1.2fr)_minmax(120px,1.8fr)_minmax(80px,1fr)_80px_80px]";
  let editingId = $state<string | null>(null);
  let editingDraft = $state<EditableFieldProperties | null>(null);
  let editingRowElement = $state<HTMLDivElement | undefined>(undefined);

  function startEditing(row: FieldRow) {
    editingId = row.id;
    editingDraft = {
      remark: row.remark,
      kind: row.kind,
      length: row.length.replace(/\s*bits$/, "")
    };
    onSelect(row.id);
  }

  function cancelEditing() {
    editingId = null;
    editingDraft = null;
  }

  function commitEditing() {
    if (!editingId || !editingDraft) return;

    onEdit(editingId, {
      remark: editingDraft.remark,
      kind: editingDraft.kind,
      length: `${editingDraft.length} bits`
    });
    cancelEditing();
  }

  function handleEditingKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      commitEditing();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      cancelEditing();
    }
  }

  function handleWindowPointerDown(event: PointerEvent) {
    if (!editingId || !editingRowElement) return;

    const target = event.target;
    if (target instanceof Node && editingRowElement.contains(target)) {
      return;
    }

    commitEditing();
  }
</script>

<svelte:window onpointerdown={handleWindowPointerDown} />

<div class="w-full min-w-0 text-[12px]" role="grid" aria-label="Protocol fields">
  <div
    class={`grid h-7 ${gridColumns} items-center gap-x-1 border-b border-border bg-foreground/[0.04] px-3 text-left font-medium text-foreground/80`}
    role="row"
  >
    <span role="columnheader">ID</span>
    <span role="columnheader">Remark</span>
    <span role="columnheader">Kind</span>
    <span role="columnheader">Length</span>
    <span role="columnheader">Offset</span>
  </div>

  {#each rows as row (row.id)}
    {#if row.id === editingId && editingDraft}
      <div
        bind:this={editingRowElement}
        class={`grid h-7 ${gridColumns} items-center gap-x-1 bg-foreground/[0.12] px-3 text-left text-foreground`}
        role="row"
        tabindex="0"
        onclick={() => onSelect(row.id)}
        onkeydown={handleEditingKeydown}
      >
        <span class="truncate">{row.id}</span>
        <input
          bind:value={editingDraft.remark}
          aria-label="Field remark"
          class="h-[22px] w-full min-w-0 rounded-sm border border-input bg-background px-1.5 text-xs text-foreground outline-none focus:border-primary focus:ring-1 focus:ring-primary/30"
        />
        <select
          bind:value={editingDraft.kind}
          aria-label="Field kind"
          style="color-scheme: dark"
          class="h-[22px] w-full min-w-0 rounded-sm border border-input bg-background px-1.5 text-xs text-foreground outline-none focus:border-primary focus:ring-1 focus:ring-primary/30"
        >
          <option>Enum</option>
          <option>Fixed</option>
          <option>Input</option>
        </select>
        <input
          bind:value={editingDraft.length}
          type="number"
          aria-label="Field length"
          class="h-[22px] w-full min-w-0 rounded-sm border border-input bg-background px-1.5 text-xs text-foreground outline-none focus:border-primary focus:ring-1 focus:ring-primary/30"
        />
        <span class="text-muted-foreground">{row.offset}</span>
      </div>
    {:else}
      <button
        type="button"
        role="row"
        aria-selected={row.id === selectedId}
        class={`grid h-7 w-full ${gridColumns} items-center gap-x-1 px-3 text-left hover:bg-foreground/[0.06] ${row.id === selectedId ? "bg-foreground/[0.12] text-foreground" : "text-foreground/90"}`}
        onclick={() => onSelect(row.id)}
        ondblclick={() => startEditing(row)}
      >
        <span class="truncate">{row.id}</span>
        <span class="truncate text-muted-foreground">{row.remark}</span>
        <span class="truncate text-muted-foreground">{row.kind}</span>
        <span class="text-muted-foreground">{row.length}</span>
        <span class="text-muted-foreground">{row.offset}</span>
      </button>
    {/if}
  {/each}
</div>

<style>
  :global(input[type="number"]::-webkit-inner-spin-button),
  :global(input[type="number"]::-webkit-outer-spin-button) {
    -webkit-appearance: none;
    margin: 0;
  }

  :global(input[type="number"]) {
    appearance: textfield;
    -moz-appearance: textfield;
  }
</style>
