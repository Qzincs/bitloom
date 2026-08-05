<script lang="ts">
  interface Props {
    value: string;
    onCommit: () => void;
    onCancel: () => void;
    placeholder?: string;
    ariaLabel?: string;
  }

  let {
    value = $bindable(),
    onCommit,
    onCancel,
    placeholder = "协议 ID",
    ariaLabel = "New protocol ID"
  }: Props = $props();

  function focusInput(node: HTMLInputElement) {
    node.focus();
  }
</script>

<input
  class="h-[22px] min-w-0 flex-1 rounded border border-ring bg-background px-1 text-[13px] outline-none"
  {placeholder}
  aria-label={ariaLabel}
  use:focusInput
  bind:value
  onkeydown={(event) => {
    if (event.key === "Enter") onCommit();
    if (event.key === "Escape") onCancel();
  }}
/>
