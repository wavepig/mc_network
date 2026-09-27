<script setup>
import { ref } from "vue";

defineProps({
  nodes: { type: Array, required: true },
  disabled: { type: Boolean, default: false },
});

const emit = defineEmits(["add", "remove", "update-node", "restore-default"]);
const draft = ref("");

function onAdd() {
  const uri = draft.value.trim();
  if (uri) {
    emit("add", uri);
    draft.value = "";
  }
}

function onEdit(index, value) {
  emit("update-node", index, value);
}
</script>

<template>
  <section class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-sm font-semibold text-stone-900">公网节点</h2>
        <p class="mt-0.5 text-xs text-stone-500">
          服务端与客户端连接时使用这些公共节点进行 P2P 中继
        </p>
      </div>
      <span class="rounded-full border border-stone-200 bg-stone-100 px-2 py-0.5 text-[11px] text-stone-600">
        {{ nodes.length }} 个节点
      </span>
    </div>

    <!-- Node list -->
    <div class="space-y-2">
      <div
        v-for="(uri, index) in nodes"
        :key="index"
        class="group flex items-center gap-2 rounded-xl border border-stone-200 bg-white px-3 py-2 shadow-soft transition-colors duration-200 hover:border-stone-300"
      >
        <span class="flex-1 text-xs font-mono text-stone-400">{{ index + 1 }}.</span>
        <input
          :value="uri"
          :disabled="disabled"
          type="text"
          class="ui-input flex-[15] px-2.5 py-1.5 font-mono"
          placeholder="tcp://host:port"
          @input="onEdit(index, $event.target.value)"
        />
        <button
          type="button"
          :disabled="disabled"
          class="cursor-pointer rounded-md px-2.5 py-1.5 text-xs text-stone-500 transition-colors duration-200 hover:bg-red-50 hover:text-red-600 disabled:cursor-not-allowed disabled:opacity-30"
          @click="emit('remove', index)"
        >
          删除
        </button>
      </div>

      <p v-if="nodes.length === 0" class="rounded-xl border border-dashed border-stone-300 py-6 text-center text-xs text-stone-500">
        暂无公网节点，点击下方添加或恢复默认
      </p>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-2 border-t border-stone-200 pt-4">
      <input
        v-model="draft"
        :disabled="disabled"
        type="text"
        class="ui-input min-w-0 flex-1 font-mono"
        placeholder="添加新节点 URI（如 tcp://public.easytier.top:11010）"
        @keyup.enter="onAdd"
      />
      <button
        type="button"
        :disabled="disabled || !draft.trim()"
        class="ui-btn-primary shrink-0"
        @click="onAdd"
      >
        添加
      </button>
      <button
        type="button"
        :disabled="disabled"
        class="ui-btn-secondary shrink-0"
        @click="emit('restore-default')"
      >
        恢复默认
      </button>
    </div>

    <p class="text-[11px] text-stone-500">修改自动保存到浏览器本地存储</p>
  </section>
</template>
