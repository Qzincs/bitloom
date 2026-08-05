<script lang="ts">
  import { untrack, type Snippet } from "svelte";
  import { cn } from "$lib/utils";

  type Direction = "horizontal" | "vertical";
  type SizedPane = "start" | "end";

  interface Props {
    direction: Direction;
    sizedPane: SizedPane;
    initialSize: number;
    minSize: number;
    minRemainingSize: number;
    label: string;
    start: Snippet;
    end: Snippet;
    collapsible?: boolean;
    collapseDistance?: number;
  }

  let {
    direction,
    sizedPane,
    initialSize,
    minSize,
    minRemainingSize,
    label,
    start,
    end,
    collapsible = false,
    collapseDistance = 0
  }: Props = $props();

  let container: HTMLDivElement;
  let size = $state(untrack(() => initialSize));
  let isCollapsed = $state(false);
  let isResizing = false;
  let resizeStartCoordinate = 0;
  let resizeStartSize = 0;
  let previousBodyCursor = "";
  let previousBodyUserSelect = "";

  let isHorizontal = $derived(direction === "horizontal");
  let trackTemplate = $derived(
    sizedPane === "start"
      ? `${size}px 0px minmax(0, 1fr)`
      : `minmax(0, 1fr) 0px ${size}px`
  );

  function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
  }

  function startResize(event: PointerEvent) {
    const divider = event.currentTarget;
    if (!(divider instanceof HTMLElement)) return;
    event.preventDefault();

    isResizing = true;
    resizeStartCoordinate = isHorizontal ? event.clientX : event.clientY;
    resizeStartSize = size;
    divider.setPointerCapture(event.pointerId);

    previousBodyCursor = document.body.style.cursor;
    previousBodyUserSelect = document.body.style.userSelect;
    document.body.style.cursor = isHorizontal ? "ew-resize" : "ns-resize";
    document.body.style.userSelect = "none";
  }

  function resize(event: PointerEvent) {
    if (!isResizing) return;

    const coordinate = isHorizontal ? event.clientX : event.clientY;
    const distance = coordinate - resizeStartCoordinate;
    const directionMultiplier = sizedPane === "start" ? 1 : -1;
    const requestedSize = resizeStartSize + distance * directionMultiplier;
    const collapseBoundary = minSize - collapseDistance;

    if (collapsible && requestedSize < collapseBoundary) {
      size = 0;
      isCollapsed = true;
      return;
    }

    const containerSize = isHorizontal ? container.clientWidth : container.clientHeight;
    const maxSize = Math.max(minSize, containerSize - minRemainingSize);
    size = clamp(requestedSize, minSize, maxSize);
    isCollapsed = false;
  }

  function stopResize(event: PointerEvent) {
    const divider = event.currentTarget;
    isResizing = false;

    document.body.style.cursor = previousBodyCursor;
    document.body.style.userSelect = previousBodyUserSelect;

    if (divider instanceof HTMLElement && divider.hasPointerCapture(event.pointerId)) {
      divider.releasePointerCapture(event.pointerId);
    }
  }
</script>

<div
  bind:this={container}
  class="grid h-full min-h-0 w-full min-w-0"
  style:grid-template-columns={isHorizontal ? trackTemplate : undefined}
  style:grid-template-rows={isHorizontal ? undefined : trackTemplate}
>
  <div class="min-h-0 min-w-0 overflow-hidden">
    {#if !(isCollapsed && sizedPane === "start")}
      {@render start()}
    {/if}
  </div>

  <div
    role="separator"
    aria-label={label}
    aria-orientation={isHorizontal ? "vertical" : "horizontal"}
    class={cn(
      "relative z-10 touch-none before:absolute before:content-[''] before:duration-100 before:delay-0",
      isHorizontal
        ? "w-[7px] -translate-x-1/2 cursor-ew-resize before:inset-y-0 before:left-1/2 before:w-px before:-translate-x-1/2 before:transition-[width,background-color] hover:before:w-1 hover:before:bg-primary hover:before:delay-300"
        : "h-[7px] -translate-y-1/2 cursor-ns-resize before:inset-x-0 before:top-1/2 before:h-px before:-translate-y-1/2 before:transition-[height,background-color] hover:before:h-1 hover:before:bg-primary hover:before:delay-300"
    )}
    onpointerdown={startResize}
    onpointermove={resize}
    onpointerup={stopResize}
    onpointercancel={stopResize}
  ></div>

  <div class="min-h-0 min-w-0 overflow-hidden">
    {#if !(isCollapsed && sizedPane === "end")}
      {@render end()}
    {/if}
  </div>
</div>
