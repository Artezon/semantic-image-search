<template>
  <Teleport to="body">
    <TransitionGroup name="modal">
      <component
        v-for="(modal, i) in modalStack"
        :key="i"
        :is="modal.component"
        v-bind="modal.props"
        :is-top="i === modalStack.length - 1"
        @close="closeModal()"
      />
    </TransitionGroup>
  </Teleport>
</template>

<script setup lang="ts">
import { modalStack, closeModal } from "./index";
</script>

<style>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-active .modal,
.modal-leave-active .modal {
  transition:
    transform 0.25s ease,
    opacity 0.25s ease;
}
.modal-enter-from .modal,
.modal-leave-to .modal {
  opacity: 0;
  transform: scale(0.9);
}
</style>
