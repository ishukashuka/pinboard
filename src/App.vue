<template>
  <div id="pinboard-app">
    <Canvas />
  </div>
</template>

<script setup lang="ts">
import Canvas from './components/Canvas.vue';
</script>

<style>
#pinboard-app {
  width: 100vw;
  height: 100vh;
  margin: 0;
  padding: 0;
  overflow: hidden;
  background-color: #121214;
  color: #e4e4e7;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  user-select: none;
}
</style>
