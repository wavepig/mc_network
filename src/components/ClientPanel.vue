<script setup>
import { computed, ref } from "vue";
import { readText, parseLinkInfo } from "../composables/useClipboard";

const props = defineProps({
  snapshot: { type: Object, required: true },
  disabled: { type: Boolean, default: false },
});

const emit = defineEmits(["create", "stop"]);

const name = ref("");
const secret = ref("");
const mcPort = ref(25565);
const pasteState = ref("idle");

const mode = computed(() => props.snapshot?.mode?.mode ?? "idle");
const errorMessage = computed(() => props.snapshot?.mode?.message ?? "未知错误");
const connecting = computed(() => mode.value === "client_connecting");
const running = computed(() => mode.value === "client_running");
const formLocked = computed(() => props.disabled || connecting.value || running.value);
const mcPortValid = computed(
  () => Number.isInteger(mcPort.value) && mcPort.value >= 1 && mcPort.value <= 65535
);
const canCreate = computed(
  () =>
    !formLocked.value &&
    name.value.trim() !== "" &&
    secret.value.trim() !== "" &&
    mcPortValid.value
);
const forwards = computed(() => props.snapshot?.forwards ?? []);

function flashPaste(state) {
  pasteState.value = state;
  setTimeout(() => {
    if (pasteState.value === state) pasteState.value = "idle";
  }, state === "ok" ? 2000 : 3000);
}

async function onPaste() {
  if (formLocked.value) return;
  let text;
  try {
    text = await readText();
  } catch {
    flashPaste("error");
    return;
  }
  const parsed = parseLinkInfo(text);
  if (!parsed) {
    flashPaste("unparsed");
    return;
  }
  name.value = parsed.name;
  secret.value = parsed.secret;
  if (parsed.port != null) mcPort.value = parsed.port;
  flashPaste("ok");
}

function onCreate() {
  if (!canCreate.value) return;
  emit("create", {
    name: name.value.trim(),
    secret: secret.value.trim(),
    mcPort: mcPort.value,
  });
}

function onStop() {
  emit("stop");
}
</script>

<template>
  <section class="ui-card p-5">
    <h2 class="text-sm font-semibold text-stone-900">客户端网络</h2>

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
          @click="onPaste"
        >
          粘贴
        </button>
      </div>

      <input
        v-model="secret"
        :disabled="formLocked"
        type="text"
        class="ui-input font-mono"
        placeholder="network-secret"
      />

      <div class="flex items-center gap-2">
        <span class="shrink-0 text-xs text-stone-500">MC 端口</span>
        <input
          v-model.number="mcPort"
          :disabled="formLocked"
          type="number"
          min="1"
          max="65535"
          class="ui-input min-w-0 flex-1 font-mono"
          placeholder="25565"
        />
      </div>

      <button
        type="button"
        :disabled="!canCreate"
        class="ui-btn-primary w-full"
        @click="onCreate"
      >
        创建客户端网络
      </button>

      <!-- Paste feedback -->
      <p v-if="pasteState === 'ok'" class="text-xs text-emerald-600">已填入 name 与 secret</p>
      <p v-else-if="pasteState === 'unparsed'" class="text-xs text-amber-700">
        粘贴内容无法解析
      </p>
      <p v-else-if="pasteState === 'error'" class="text-xs text-red-600">剪贴板读取失败</p>

      <p v-if="disabled" class="text-xs text-amber-700">
        服务端网络运行中，先关闭后才能创建客户端网络
      </p>
      <template v-else-if="!formLocked">
        <p v-if="!name.trim() || !secret.trim()" class="text-xs text-stone-500">
          name 与 secret 均不能为空，可粘贴连接信息或手动输入
        </p>
        <p v-if="!mcPortValid" class="text-xs text-amber-700">MC 端口需为 1–65535 的整数</p>
      </template>
    </div>

    <!-- Connecting -->
    <div
      v-if="connecting"
      class="mt-4 rounded-xl border border-amber-200 bg-amber-50/70 p-3 text-xs"
    >
      <div class="flex items-center gap-2 text-amber-800">
        <span class="inline-block h-3 w-3 animate-spin rounded-full border-2 border-amber-200 border-t-amber-600" />
        连接中…
      </div>
      <p class="mt-1.5 text-stone-500">peer 验证与端口转发进行中</p>
      <p class="mt-1 text-stone-500">启动 core → peer 验证 → 端口转发 → LAN 广播</p>
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
        <p class="text-sm font-medium text-emerald-600">客户端网络运行中</p>
        <p class="mt-2 text-xs text-stone-600">
          打开 MC → 多人游戏 → 局域网游戏，点击进入
        </p>
        <p class="mt-1 text-xs text-stone-500">
          或直接连接 <span class="font-mono text-stone-800">127.0.0.1:{{ snapshot?.mc_port }}</span>
        </p>

        <!-- Forward table -->
        <div class="mt-3">
          <p class="text-[11px] font-medium text-stone-500">端口转发明细</p>
          <table v-if="forwards.length" class="mt-1.5 w-full text-left text-xs">
            <thead>
              <tr class="text-stone-500">
                <th class="pb-1 font-medium">协议</th>
                <th class="pb-1 font-medium">本地</th>
                <th class="pb-1 font-medium">远端</th>
                <th class="pb-1 font-medium">状态</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(item, index) in forwards" :key="index" class="text-stone-600">
                <td class="py-0.5 font-mono">{{ item.proto }}</td>
                <td class="truncate py-0.5 font-mono">{{ item.local }}</td>
                <td class="truncate py-0.5 font-mono">{{ item.remote }}</td>
                <td
                  class="py-0.5"
                  :class="item.ok ? 'text-emerald-600' : 'text-red-600'"
                >
                  {{ item.ok ? "成功" : "失败" }}
                </td>
              </tr>
            </tbody>
          </table>
          <p v-else class="mt-1 text-xs text-stone-500">暂无转发明细</p>
        </div>
      </template>

      <button
        type="button"
        class="ui-btn-danger mt-3"
        @click="onStop"
      >
        关闭客户端网络
      </button>
    </div>
  </section>
</template>
