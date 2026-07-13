<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let imageData: ImageData;
  let rafId = 0;
  let running = true;

  async function setupCanvas() {
    const { width, height } = await invoke<{ width: number; height: number }>(
      "get_screen_info",
    );
    canvasEl.width = width;
    canvasEl.height = height;
    ctx = canvasEl.getContext("2d", { alpha: false })!;
    imageData = ctx.createImageData(width, height);
  }

  async function frameLoop() {
    if (!running) return;
    try {
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
    running = false;
    cancelAnimationFrame(rafId);
  });
</script>

<canvas bind:this={canvasEl}></canvas>

<style>
  canvas {
    image-rendering: pixelated;
    width: 100%;
    height: auto;
  }
</style>
