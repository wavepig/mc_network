<script setup>
import { computed, nextTick, ref, watch } from "vue";

const props = defineProps({
  snapshot: { type: Object, required: true },
  logs: { type: Array, default: () => [] },
});

const MODE_META = {
  idle: { label: "空闲", cls: "bg-stone-100 text-stone-500 border-stone-200" },
  scanning: { label: "嗅探中", cls: "bg-amber-50 text-amber-700 border-amber-200" },
  server_running: {
    label: "服务端运行中",
    cls: "bg-emerald-50 text-emerald-700 border-emerald-200",
  },
  client_connecting: {
    label: "客户端连接中",
    cls: "bg-amber-50 text-amber-700 border-amber-200",
  },
  client_running: {
    label: "客户端运行中",
    cls: "bg-emerald-50 text-emerald-700 border-emerald-200",
  },
  error: { label: "异常", cls: "bg-red-50 text-red-700 border-red-200" },
};

const mode = computed(() => props.snapshot?.mode?.mode ?? "idle");
const modeMeta = computed(() => MODE_META[mode.value] ?? MODE_META.idle);
const errorMessage = computed(() => props.snapshot?.mode?.message ?? "");
const mcPort = computed(() => props.snapshot?.mc_port ?? null);
const rpcPort = computed(() => props.snapshot?.rpc_port ?? null);
const portText = computed(() => {
  const parts = [];
  if (mcPort.value != null) parts.push(`MC ${mcPort.value}`);
  if (rpcPort.value != null) parts.push(`RPC ${rpcPort.value}`);
  return parts.join(" · ");
});
const lines = computed(() => props.logs ?? []);

// 纯展示：按日志文本关键字着色左边条
function lineTone(line) {
  const s = String(line);
  if (/error|failed|failure|panic|错误|失败|异常/i.test(s)) {
    return "border-l-red-300 text-red-700";
  }
  if (/warn|警告|注意/i.test(s)) {
    return "border-l-amber-300 text-amber-700";
  }
  if (/success|started|listening|connected|成功|启动|已连接|运行/i.test(s)) {
    return "border-l-emerald-300 text-emerald-700";
  }
  return "border-l-stone-200 text-stone-600";
}

const logBox = ref(null);

watch(
  () => props.logs,
  async () => {
    await nextTick();
    if (logBox.value) logBox.value.scrollTop = logBox.value.scrollHeight;
  },
  { deep: true }
);
</script>

<template>
  <section class="ui-card p-5">
    <h2 class="text-sm font-semibold text-stone-900">状态与日志</h2>

    <div class="mt-3 flex flex-wrap items-center gap-2">
      <span
        class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-[11px] font-medium"
        :class="modeMeta.cls"
      >
        <span class="h-1.5 w-1.5 rounded-full bg-current" />
        {{ modeMeta.label }}
      </span>
      <span v-if="portText" class="font-mono text-[11px] text-stone-500">{{ portText }}</span>
      <span v-if="mode === 'error' && errorMessage" class="text-xs text-red-600">
        {{ errorMessage }}
      </span>
    </div>

    <div
      ref="logBox"
      class="mt-3 max-h-40 overflow-y-auto rounded-xl border border-stone-200 bg-stone-50 p-3 font-mono text-[11px] leading-relaxed"
    >
      <p v-if="lines.length === 0" class="text-stone-400">暂无日志</p>
      <p
        v-for="(line, i) in lines"
        :key="i"
        class="border-l-2 py-0.5 pl-2 whitespace-pre-wrap break-all"
        :class="lineTone(line)"
      >
        {{ line }}
      </p>
    </div>
  </section>
</template>
