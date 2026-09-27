<script setup>
import { computed, ref } from "vue";
import { randomName, randomSecret } from "../utils/idGen";
import { copyText, formatLinkInfo } from "../composables/useClipboard";

const props = defineProps({
  snapshot: { type: Object, required: true },
  scan: { type: Object, required: true },
  disabled: { type: Boolean, default: false },
});

const emit = defineEmits(["create", "stop"]);

const name = ref("");
const secret = ref("");
const copyState = ref("idle");

const mode = computed(() => props.snapshot?.mode?.mode ?? "idle");
const errorMessage = computed(() => props.snapshot?.mode?.message ?? "未知错误");
const scanning = computed(() => mode.value === "scanning");
const running = computed(() => mode.value === "server_running");
const formLocked = computed(() => props.disabled || scanning.value || running.value);
const canCreate = computed(
  () => !formLocked.value && name.value.trim() !== "" && secret.value.trim() !== ""
);

function onRandomName() {
  name.value = randomName();
}

function onRandomSecret() {
  secret.value = randomSecret();
}

function onCreate() {
  if (!canCreate.value) return;
  emit("create", { name: name.value.trim(), secret: secret.value.trim() });
}

async function onCopy() {
  try {
    await copyText(
      formatLinkInfo(
        props.snapshot?.name ?? "",
        props.snapshot?.secret ?? "",
        props.snapshot?.mc_port ?? ""
      )
    );
    copyState.value = "copied";
    setTimeout(() => {
      if (copyState.value === "copied") copyState.value = "idle";
    }, 2000);
  } catch {
    copyState.value = "failed";
    setTimeout(() => {
      if (copyState.value === "failed") copyState.value = "idle";
    }, 3000);
  }
}

function onStop() {
  emit("stop");
}
</script>

<template>
  <section class="ui-card p-5">
    <h2 class="text-sm font-semibold text-stone-900">服务端网络</h2>

    <!-- Form -->
    <div class="mt-4 space-y-2.5">
      <div class="flex items-center gap-2">
        <input
          v-model="name"
          :disabled="formLocked"
          type="text"
          class="ui-input min-w-0 flex-1 font-mono"
          placeholder="network-name"
        />
        <button
          type="button"
          :disabled="formLocked"
          class="ui-btn-secondary shrink-0 px-3 py-2 text-xs"
          @click="onRandomName"
        >
          随机
        </button>
      </div>

      <div class="flex items-center gap-2">
        <input
          v-model="secret"
          :disabled="formLocked"
          type="text"
          class="ui-input min-w-0 flex-1 font-mono"
          placeholder="network-secret"
        />
        <button
          type="button"
          :disabled="formLocked"
          class="ui-btn-secondary shrink-0 px-3 py-2 text-xs"
          @click="onRandomSecret"
        >
          随机
        </button>
      </div>

      <button
        type="button"
        :disabled="!canCreate"
        class="ui-btn-primary w-full"
        @click="onCreate"
      >
        创建服务端网络
      </button>

      <p v-if="disabled" class="text-xs text-amber-700">
        客户端网络运行中，先关闭后才能创建服务端网络
      </p>
      <p v-else-if="!formLocked && (!name.trim() || !secret.trim())" class="text-xs text-stone-500">
        name 与 secret 均不能为空，可点「随机」生成
      </p>
    </div>

    <!-- Scanning -->
    <div
      v-if="scanning"
      class="mt-4 rounded-xl border border-amber-200 bg-amber-50/70 p-3 text-xs"
    >
      <div class="flex items-center gap-2 text-amber-800">
        <span class="inline-block h-3 w-3 animate-spin rounded-full border-2 border-amber-200 border-t-amber-600" />
        嗅探 MC 服务器中…（最长 30 秒）
      </div>
      <p class="mt-1.5 text-stone-500">请先在 MC 中「对局域网开放」</p>
      <p v-if="scan?.hit && scan?.port" class="mt-1 font-medium text-emerald-600">
        已发现端口 {{ scan.port }}
      </p>
    </div>

    <!-- Error -->
    <p
      v-if="mode === 'error'"
      class="mt-4 rounded-xl border border-red-200 bg-red-50 p-3 text-xs text-red-700"
    >
      {{ errorMessage }}
    </p>

    <!-- Result / Stop -->
    <div
      v-if="running || mode === 'error'"
      class="mt-4 rounded-xl border border-stone-200 bg-stone-50/80 p-3"
    >
      <template v-if="running">
        <div class="space-y-1.5 text-xs">
          <div class="flex items-center justify-between">
            <span class="text-stone-500">网络名称</span>
            <span class="font-mono text-stone-800">{{ snapshot?.name }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-stone-500">网络密码</span>
            <span class="font-mono text-stone-800">{{ snapshot?.secret }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-stone-500">主机名</span>
            <span class="font-mono text-stone-800">{{ snapshot?.hostname }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-stone-500">MC 端口</span>
            <span class="font-mono text-stone-800">{{ snapshot?.mc_port }}</span>
          </div>
        </div>
      </template>

      <div class="mt-3 flex items-center gap-2">
        <button
          v-if="running"
          type="button"
          class="ui-btn-info"
          @click="onCopy"
        >
          复制连接信息
        </button>
        <button
          type="button"
          class="ui-btn-danger"
          @click="onStop"
        >
          关闭服务端网络
        </button>
        <span v-if="copyState === 'copied'" class="text-xs text-emerald-600">已复制</span>
        <span v-else-if="copyState === 'failed'" class="text-xs text-red-600">复制失败</span>
      </div>
    </div>
  </section>
</template>
