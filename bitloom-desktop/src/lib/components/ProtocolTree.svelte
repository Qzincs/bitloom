<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ProtocolTreeItem from "$lib/components/ProtocolTreeItem.svelte";
  import ProtocolCreateInput from "$lib/components/ProtocolCreateInput.svelte";
  import ContextMenu, {
    type ContextMenuItem
  } from "$lib/components/ui/ContextMenu.svelte";
  import {
    containsId,
    findNode,
    insertNode,
    removeNode,
    updateNode,
    type ProtocolNode
  } from "$lib/components/protocol-tree";

  interface Props {
    onSelectionChange?: (id: string | null) => void;
  }

  let { onSelectionChange }: Props = $props();

  const workbenchFont =
    'system-ui, Ubuntu, "Droid Sans", "Source Han Sans SC", "Source Han Sans CN", "Source Han Sans", sans-serif';
  interface BackendProtocolTreeNode {
    id: string;
    remark: string | null;
    children: BackendProtocolTreeNode[];
  }

  let protocolNodes = $state<ProtocolNode[]>([]);

  function toProtocolNode(node: BackendProtocolTreeNode): ProtocolNode {
    return {
      id: node.id,
      remark: node.remark ?? node.id,
      children: node.children.map(toProtocolNode)
    };
  }

  async function loadProtocolTree() {
    try {
      const nodes = await invoke<BackendProtocolTreeNode[]>("get_protocol_tree");
      protocolNodes = nodes.map(toProtocolNode);
    } catch (error) {
      console.error("Failed to load protocol tree", error);
    }
  }

  onMount(() => {
    void loadProtocolTree();
  });

  let selectedProtocol = $state<string | null>(null);
  let isCreating = $state(false);
  let isCreatingProtocol = $state(false);
  let draftParentId = $state<string | null>(null);
  let draftId = $state("");
  let draftError = $state("");
  let renamingId = $state<string | null>(null);
  let renameDraft = $state("");
  let contextMenu = $state({
    open: false,
    x: 0,
    y: 0,
    targetId: null as string | null
  });
  const protocolContextMenuItems: ContextMenuItem[] = [
    { id: "add-child", label: "新增子协议" },
    { id: "rename", label: "编辑备注" },
    { id: "delete", label: "删除" }
  ];
  let draftValidationError = $derived(
    draftId.trim() && containsId(protocolNodes, draftId.trim())
      ? "这个协议 ID 已存在"
      : draftError
  );

  function setSelectedProtocol(protocol: string | null) {
    selectedProtocol = protocol;
    onSelectionChange?.(protocol);
  }

  function selectProtocol(protocol: string) {
    setSelectedProtocol(protocol);
  }

  function startRenaming(protocolId: string) {
    const node = findNode(protocolNodes, protocolId);
    if (!node) return;

    setSelectedProtocol(protocolId);
    renamingId = protocolId;
    renameDraft = node.remark;
  }

  function cancelRenaming() {
    renamingId = null;
    renameDraft = "";
  }

  function commitRenaming() {
    if (!renamingId) return;

    const id = renamingId;
    const remark = renameDraft.trim() || id;
    protocolNodes = updateNode(protocolNodes, id, (node) => ({
      ...node,
      remark
    }));
    cancelRenaming();
  }

  function deleteProtocol(protocolId: string) {
    const node = findNode(protocolNodes, protocolId);
    if (!node) return;

    const hasChildren = (node.children?.length ?? 0) > 0;
    const message = hasChildren
      ? `确定删除协议“${node.id}”及其所有子协议吗？`
      : `确定删除协议“${node.id}”吗？`;

    if (!window.confirm(message)) return;

    protocolNodes = removeNode(protocolNodes, protocolId);
    if (selectedProtocol && !containsId(protocolNodes, selectedProtocol)) {
      setSelectedProtocol(null);
    }
    if (renamingId === protocolId) {
      cancelRenaming();
    }
  }

  function openProtocolContextMenu(event: MouseEvent, protocolId: string) {
    event.preventDefault();
    contextMenu = {
      open: true,
      x: event.clientX,
      y: event.clientY,
      targetId: protocolId
    };
  }

  function closeContextMenu() {
    contextMenu.open = false;
  }

  function selectContextMenuItem(itemId: string) {
    const targetId = contextMenu.targetId;
    closeContextMenu();

    if (itemId === "add-child" && targetId) {
      setSelectedProtocol(targetId);
      startCreating();
    }
    if (itemId === "rename" && targetId) {
      startRenaming(targetId);
    }
    if (itemId === "delete" && targetId) {
      deleteProtocol(targetId);
    }
  }

  function startCreating() {
    if (isCreating) return;

    draftParentId = selectedProtocol;
    draftId = "";
    draftError = "";
    isCreating = true;
  }

  function cancelCreating() {
    isCreating = false;
    draftId = "";
    draftError = "";
  }

  async function commitCreating() {
    if (isCreatingProtocol) return;

    const id = draftId.trim();
    if (!id) {
      draftError = "请输入协议 ID";
      return;
    }
    if (containsId(protocolNodes, id)) {
      return;
    }

    isCreatingProtocol = true;
    try {
      await invoke("create_protocol", {
        id,
        parentId: draftParentId
      });

      const newNode = { id, remark: id };
      protocolNodes = insertNode(protocolNodes, draftParentId, newNode);
      setSelectedProtocol(id);
      cancelCreating();
    } catch (error) {
      draftError = error instanceof Error ? error.message : String(error);
    } finally {
      isCreatingProtocol = false;
    }
  }

  function finishCreating() {
    const id = draftId.trim();
    if (!id || containsId(protocolNodes, id)) {
      cancelCreating();
    } else {
      commitCreating();
    }
  }

  function handleWindowPointerDown(event: PointerEvent) {
    const target = event.target;
    if (isCreating) {
      if (target instanceof Element && target.closest("[data-protocol-create]")) {
        return;
      }
      finishCreating();
    }
    if (renamingId) {
      if (target instanceof Element && target.closest("[data-protocol-rename]")) {
        return;
      }
      commitRenaming();
    }
  }

  function handleWindowBlur() {
    if (isCreating) {
      finishCreating();
    }
    if (renamingId) {
      commitRenaming();
    }
  }
</script>

{#snippet addIcon()}
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
    aria-hidden="true"
  >
    <path
      d="M8 1.5C8 1.22386 7.77614 1 7.5 1C7.22386 1 7 1.22386 7 1.5V7H1.5C1.22386 7 1 7.22386 1 7.5C1 7.77614 1.22386 8 1.5 8H7V13.5C7 13.7761 7.22386 14 7.5 14C7.77614 14 8 13.7761 8 13.5V8H13.5C13.7761 8 14 7.77614 14 7.5C14 7.22386 13.7761 7 13.5 7H8V1.5Z"
    />
  </svg>
{/snippet}

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onblur={handleWindowBlur}
/>

<aside
  class="group/tree flex h-full min-h-0 flex-col border-r border-border bg-card"
  style:font-family={workbenchFont}
>
  <header
    class="flex h-10 shrink-0 items-center justify-between px-3 text-[13px] font-semibold"
  >
    <span>Protocols</span>
    <button
      type="button"
      aria-label="Add protocol"
      data-protocol-create
      class="rounded p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
      onclick={startCreating}
    >
      {@render addIcon()}
    </button>
  </header>

  <div class="flex h-[22px] shrink-0 items-center gap-1 px-2 font-sans text-[11px] font-bold">
    <span class="truncate">BITLOOM-TAURI</span>
  </div>

  <div class="protocol-tree-scroll min-h-0 flex-1 overflow-auto pb-3 text-[13px]">
    <div class="ml-2 pl-2">
      {#if isCreating && draftParentId === null}
        <div class="ml-5 flex h-[22px] items-center gap-1" data-protocol-create>
          <ProtocolCreateInput
            bind:value={draftId}
            onCommit={commitCreating}
            onCancel={cancelCreating}
          />
        </div>
        {#if draftValidationError}
          <p class="px-1 text-[11px] text-destructive">{draftValidationError}</p>
        {/if}
      {/if}

      {#each protocolNodes as node (node.id)}
        <ProtocolTreeItem
          {node}
          depth={0}
          selectedId={selectedProtocol}
          onSelect={selectProtocol}
          onContextMenu={openProtocolContextMenu}
          renamingId={renamingId}
          bind:renameDraft
          onRenameCommit={commitRenaming}
          onRenameCancel={cancelRenaming}
          ancestorActive={[]}
          {isCreating}
          {draftParentId}
          bind:draftId
          draftError={draftValidationError}
          onDraftCommit={commitCreating}
          onDraftCancel={cancelCreating}
        />
      {/each}
    </div>
  </div>

  <ContextMenu
    open={contextMenu.open}
    x={contextMenu.x}
    y={contextMenu.y}
    items={protocolContextMenuItems}
    onSelect={selectContextMenuItem}
    onClose={closeContextMenu}
  />
</aside>

<style>
  :global(.protocol-tree-scroll) {
    scrollbar-width: auto;
  }

  :global(.protocol-tree-scroll::-webkit-scrollbar) {
    -webkit-appearance: none;
    width: 10px;
    height: 10px;
    background: transparent;
  }

  :global(.protocol-tree-scroll::-webkit-scrollbar-track) {
    -webkit-appearance: none;
    background: transparent;
  }

  :global(.protocol-tree-scroll::-webkit-scrollbar-thumb) {
    -webkit-appearance: none;
    background-color: transparent;
    background-clip: border-box;
    border: 0;
    border-radius: 0;
    transition: background-color 220ms ease, opacity 280ms ease;
  }

  :global(.protocol-tree-scroll:hover::-webkit-scrollbar-thumb) {
    background-color: rgb(255 255 255 / 0.18);
  }

  :global(.protocol-tree-scroll:hover::-webkit-scrollbar-thumb:hover) {
    background-color: rgb(255 255 255 / 0.36);
  }

  :global(.protocol-tree-scroll:hover::-webkit-scrollbar-thumb:active) {
    background-color: rgb(255 255 255 / 0.52);
  }
</style>
