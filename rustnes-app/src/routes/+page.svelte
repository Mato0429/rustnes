<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  const appWindow = getCurrentWindow();
  const pressedKeys = new Set();

  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let imageData: ImageData;
  let currentkey: KeyboardEvent;
  let rafId = 0;

  async function setupCanvas() {
    const width = 256;
    const height = 240;
    canvasEl.width = width;
    canvasEl.height = height;
    ctx = canvasEl.getContext("2d", { alpha: false })!;
    imageData = ctx.createImageData(width, height);
  }

  async function openCartDialog() {
    const path = await open({
      multiple: false,
      filters: [{ name: "NES Cartridge", extensions: ["nes"] }],
    });

    if (!path) return;
    await invoke("load_cart", { path: path });
  }

  async function frameLoop() {
    try {
      let a = +pressedKeys.has("KeyJ") || +pressedKeys.has("KeyZ");
      let b = (+pressedKeys.has("KeyK") || +pressedKeys.has("KeyX"))<< 1;
      let select = +pressedKeys.has("KeyF") << 2;
      let start = +pressedKeys.has("KeyH") << 3;

      let up = (+pressedKeys.has("KeyW") || +pressedKeys.has("ArrowUp")) << 4;
      let down = (+pressedKeys.has("KeyS") || +pressedKeys.has("ArrowDown"))<< 5;
      let left = (+pressedKeys.has("KeyA") || +pressedKeys.has("ArrowLeft")) << 6;
      let right = (+pressedKeys.has("KeyD") || +pressedKeys.has("ArrowRight"))<< 7;

      let joypad1 = right | left | down | up | start | select | b | a;
      await invoke("update_joypad1", { v: joypad1 });

      const buf = await invoke<ArrayBuffer>("get_frame");
      imageData.data.set(new Uint8ClampedArray(buf));
      ctx.putImageData(imageData, 0, 0);
    } catch (e) {
      console.error("frame fetch failed", e);
    }
    rafId = requestAnimationFrame(frameLoop);
  }

  onMount(async () => {
    await setupCanvas();
    rafId = requestAnimationFrame(frameLoop);
  });

  onDestroy(() => {
    cancelAnimationFrame(rafId);
  });
</script>

<svelte:window
  onkeydown={(e) => pressedKeys.add(e.code)}
  onkeyup={(e) => pressedKeys.delete(e.code)}
/>

<div class="app-container">
  <div class="titlebar">
    <div data-tauri-drag-region class="drag-region">
      <!-- <img class="titlebar-icon" alt="" /> -->
      <span class="titlebar-title">rustnes</span>
    </div>
    <div class="controls">
      <button
        id="titlebar-minimize"
        title="minimize"
        onclick={async () => await appWindow.minimize()}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
        >
          <path d="M19 13H5v-1h14z" />
        </svg></button
      >
      <button
        id="titlebar-maximize"
        title="maximize"
        onclick={async () => await appWindow.toggleMaximize()}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
        >
          <path d="M8 8v8h8V8Zm9-1V17H7V7Z" />
        </svg>
      </button>

      <button
        id="titlebar-close"
        title="close"
        onclick={async () => await appWindow.close()}
        ><svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
        >
          <path d="M8 7l9 9v1H16L7 8V7ZM7 17V16l9-9h1V8L8 17Z" />
        </svg></button
      >
    </div>
  </div>

  <div class="menubar">
    <div class="controls">
      <button class="menubar-file" onclick={openCartDialog}>File</button>
    </div>
  </div>

  <div class="content">
    <canvas class="display" bind:this={canvasEl}></canvas>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    overflow: hidden;
  }

  .app-container {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar {
    flex: 0;
    height: 30px;
    background: #ffffff;
    display: grid;
    grid-template-columns: auto max-content;
  }

  .titlebar > .drag-region {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-left: 8px;
    min-width: 0;
  }

  .titlebar-title {
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }

  .titlebar > .controls {
    display: flex;
  }

  .titlebar button {
    appearance: none;
    border: none;
    display: inline-flex;
    justify-content: center;
    align-items: center;
    width: 45px;
    height: 100%;
    background-color: transparent;
    transition: background-color 0.1s ease;
    fill: black;
  }

  #titlebar-minimize:hover,
  #titlebar-maximize:hover {
    background-color: lightgray;
  }

  #titlebar-close:hover {
    background-color: #e81123;
    fill: white;
  }

  .menubar {
    flex: 0;
    margin-left: 5px;
    height: 25px;
    background: #ffffff;
    display: grid;
    grid-template-columns: auto max-content;
  }

  .menubar > .controls {
    display: flex;
  }

  .menubar button {
    appearance: none;
    border: none;
    display: inline-flex;
    justify-content: center;
    align-items: center;
    padding-left: 10px;
    padding-right: 10px;
    height: 90%;
    background-color: transparent;
    transition: background-color 0.1s ease;
    fill: black;
  }

  .menubar button:hover {
    background-color: lightgrey;
    fill: white;
  }

  .content {
    flex: 1;
    min-height: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #1f1f1f;
  }

  .display {
    image-rendering: pixelated;
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
</style>
