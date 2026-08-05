<script lang="ts">
  import FieldTable, {
    type FieldRow
  } from "$lib/components/protocol/FieldTable.svelte";
  import ProtocolTree from "$lib/components/ProtocolTree.svelte";
  import ResizableSplit from "$lib/components/ResizableSplit.svelte";

  let selectedProtocolId = $state<string | null>(null);
  let selectedFieldId = $state<string | null>(null);

  const protocolRemarks: Record<string, string> = {
    ethernet: "Ethernet II",
    ipv4: "IPv4",
    ipv6: "IPv6"
  };

  let exampleFields = $state<FieldRow[]>([
    { id: "dst_mac", remark: "Destination MAC", kind: "Input", length: "48 bits", offset: "0" },
    { id: "src_mac", remark: "Source MAC", kind: "Input", length: "48 bits", offset: "48" },
    { id: "ether_type", remark: "Payload protocol", kind: "Enum", length: "16 bits", offset: "96" }
  ]);

  function editField(id: string, patch: Partial<FieldRow>) {
    exampleFields = exampleFields.map((field) =>
      field.id === id ? { ...field, ...patch } : field
    );
  }
</script>

<svelte:head>
  <title>BitLoom</title>
</svelte:head>

{#snippet protocolsPane()}
  <ProtocolTree
    onSelectionChange={(id) => {
      selectedProtocolId = id;
    }}
  />
{/snippet}

{#snippet contentPane()}
  <section class="h-full min-w-0 p-4">
    {#if selectedProtocolId}
      <header class="mb-5">
        <h2 class="text-lg font-semibold">
          {selectedProtocolId}
          {#if protocolRemarks[selectedProtocolId]}
            <span class="ml-1 font-normal text-muted-foreground">
              ({protocolRemarks[selectedProtocolId]})
            </span>
          {/if}
        </h2>
        <p class="mt-1 text-sm text-muted-foreground">
          {exampleFields.length} fields · Big Endian
        </p>
      </header>
    {:else}
      <h2 class="text-lg font-semibold">Content</h2>
      <p class="mt-2 text-sm text-muted-foreground">No protocol selected</p>
    {/if}

    {#if selectedProtocolId}
      <div class="-mx-4 min-w-0 overflow-auto border-y border-border">
        <FieldTable
          rows={exampleFields}
          selectedId={selectedFieldId}
          onSelect={(id) => {
            selectedFieldId = id;
          }}
          onEdit={editField}
        />
      </div>
    {/if}
  </section>
{/snippet}

{#snippet contentHexPane()}
  <ResizableSplit
    direction="vertical"
    sizedPane="end"
    initialSize={180}
    minSize={96}
    minRemainingSize={160}
    collapsible
    collapseDistance={80}
    label="Resize Hex View"
    start={contentPane}
    end={hexPane}
  />
{/snippet}

{#snippet inspectorPane()}
  <section class="h-full border-l border-border bg-card p-4">
    <h2 class="text-lg font-semibold">Inspector</h2>
  </section>
{/snippet}

{#snippet workspacePane()}
  <ResizableSplit
    direction="horizontal"
    sizedPane="end"
    initialSize={280}
    minSize={200}
    minRemainingSize={320}
    collapsible
    collapseDistance={80}
    label="Resize Inspector"
    start={contentHexPane}
    end={inspectorPane}
  />
{/snippet}

{#snippet hexPane()}
  <section class="h-full border-t border-border bg-card p-4">Hex View</section>
{/snippet}

<main
  class="grid h-screen grid-rows-[44px_minmax(0,1fr)] bg-background text-foreground"
>
  <header
    class="flex items-center border-b border-border bg-card px-4 font-semibold"
  >
    Toolbar
  </header>

  <ResizableSplit
    direction="horizontal"
    sizedPane="start"
    initialSize={240}
    minSize={160}
    minRemainingSize={320}
    collapsible
    collapseDistance={80}
    label="Resize Protocols"
    start={protocolsPane}
    end={workspacePane}
  />
</main>
