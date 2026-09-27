<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import RoleSelect from "./components/RoleSelect.vue";
import PublicNodesPanel from "./components/PublicNodesPanel.vue";
import ServerPanel from "./components/ServerPanel.vue";
import ClientPanel from "./components/ClientPanel.vue";
import StatusLogPanel from "./components/StatusLogPanel.vue";
import { startClient, startServer, stopNetwork, useNetwork } from "./composables/useNetwork";
import { usePublicNodes } from "./composables/usePublicNodes";

const { snapshot, logs, scan, refresh } = useNetwork();
const { nodes, add, remove, restoreDefault } = usePublicNodes();

const activeTab = ref("home");
const homeView = ref("select"); // select | server | client
const iconPreview = ref(false);

const serverError = ref("");
const clientError = ref("");

const MODE_LABELS = {
  idle: "空闲",
  scanning: "嗅探中",
  server_running: "服务端运行中",
  client_connecting: "客户端连接中",
  client_running: "客户端运行中",
  error: "异常",
};

const mode = computed(() => snapshot.value?.mode?.mode ?? "idle");
const modeLabel = computed(() => MODE_LABELS[mode.value] ?? "空闲");
const busy = computed(() => !["idle", "error"].includes(mode.value));
const isServer = computed(() => ["scanning", "server_running"].includes(mode.value));
const isClient = computed(() => ["client_connecting", "client_running"].includes(mode.value));

function onKeydown(e) {
  if (e.key === "Escape") iconPreview.value = false;
}

onMounted(() => {
  refresh();
  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

function nodeUris() {
  return nodes.value.map((uri) => uri.trim()).filter(Boolean);
}

function onUpdateNode(index, value) {
  nodes.value[index] = value;
}

async function onCreateServer({ name, secret }) {
  serverError.value = "";
  const list = nodeUris();
  if (list.length === 0) {
    serverError.value = "至少需要一个公网节点";
    return;
  }
  try {
    await startServer({ name, secret, publicNodes: list });
    await refresh();
  } catch (e) {
    serverError.value = String(e);
  }
}

async function onCreateClient({ name, secret, mcPort }) {
  clientError.value = "";
  const list = nodeUris();
  if (list.length === 0) {
    clientError.value = "至少需要一个公网节点";
    return;
  }
  try {
    await startClient({ name, secret, publicNodes: list, mcPort });
    await refresh();
  } catch (e) {
    clientError.value = String(e);
  }
}

async function onStop(which) {
  serverError.value = "";
  clientError.value = "";
  try {
    await stopNetwork();
    await refresh();
  } catch (e) {
    if (which === "server") serverError.value = String(e);
    else clientError.value = String(e);
  }
}
</script>

<template>
  <div class="relative flex min-h-screen flex-col bg-page text-stone-800">
    <!-- Ambient sun glow -->
    <div
      class="pointer-events-none fixed inset-x-0 top-0 z-0 h-72 bg-gradient-to-b from-amber-200/50 via-orange-100/20 to-transparent"
      aria-hidden="true"
    />

    <!-- Header -->
    <header class="sticky top-0 z-30 border-b border-stone-200/80 bg-white/85 backdrop-blur">
      <div class="mx-auto flex h-12 max-w-5xl items-center justify-between px-4">
        <h1 class="flex items-center gap-2 text-sm font-semibold tracking-wide text-stone-900">
          <button
            type="button"
            class="cursor-pointer rounded-md transition-opacity duration-200 hover:opacity-80 focus-visible:ring-2 focus-visible:ring-amber-500/40 focus-visible:outline-none"
            aria-label="查看应用图标"
            @click="iconPreview = true"
          >
            <img src="/icon.png" alt="" class="h-6 w-6 rounded-md" />
          </button>
          mc-network
        </h1>
        <span
          class="rounded-full border px-2.5 py-0.5 text-[11px] font-medium"
          :class="
            mode === 'server_running' || mode === 'client_running'
              ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
              : mode === 'scanning' || mode === 'client_connecting'
                ? 'border-amber-200 bg-amber-50 text-amber-700'
                : mode === 'error'
                  ? 'border-red-200 bg-red-50 text-red-700'
                  : 'border-stone-200 bg-stone-100 text-stone-500'
          "
        >
          <span
            v-if="mode === 'server_running' || mode === 'client_running'"
            class="mr-1 inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500 align-middle"
            aria-hidden="true"
          />
          {{ modeLabel }}
        </span>
      </div>

      <!-- Tabs -->
      <nav class="mx-auto flex max-w-5xl items-center gap-1.5 px-4 pb-2.5" aria-label="主导航">
        <button
          type="button"
          class="cursor-pointer rounded-full px-4 py-1.5 text-sm transition-colors duration-200"
          :class="activeTab === 'home' ? 'bg-amber-100 font-medium text-amber-900' : 'text-stone-500 hover:bg-stone-100 hover:text-stone-700'"
          @click="activeTab = 'home'"
        >
          首页
        </button>
        <button
          type="button"
          class="cursor-pointer rounded-full px-4 py-1.5 text-sm transition-colors duration-200"
          :class="activeTab === 'settings' ? 'bg-amber-100 font-medium text-amber-900' : 'text-stone-500 hover:bg-stone-100 hover:text-stone-700'"
          @click="activeTab = 'settings'"
        >
          设置
        </button>
      </nav>
    </header>

    <!-- Content -->
    <main class="relative z-10 mx-auto w-full max-w-5xl flex-1 px-4 py-5">
      <!-- Home tab -->
      <div v-if="activeTab === 'home'">
        <Transition name="view" mode="out-in">
          <!-- Role selection -->
          <RoleSelect v-if="homeView === 'select'" key="select" @select="homeView = $event" />

          <!-- Server creation -->
          <div v-else-if="homeView === 'server'" key="server" class="space-y-4">
            <button
              type="button"
              class="flex cursor-pointer items-center gap-1 text-sm text-stone-500 transition-colors duration-200 hover:text-amber-700"
              @click="homeView = 'select'"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
              </svg>
              返回选择
            </button>

            <p
              v-if="serverError"
              class="rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700"
            >
              {{ serverError }}
            </p>

            <ServerPanel
              :snapshot="snapshot"
              :scan="scan"
              :disabled="isClient"
              @create="onCreateServer"
              @stop="onStop('server')"
            />
            <StatusLogPanel :snapshot="snapshot" :logs="logs" />
          </div>

          <!-- Client creation -->
          <div v-else key="client" class="space-y-4">
            <button
              type="button"
              class="flex cursor-pointer items-center gap-1 text-sm text-stone-500 transition-colors duration-200 hover:text-amber-700"
              @click="homeView = 'select'"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
              </svg>
              返回选择
            </button>

            <p
              v-if="clientError"
              class="rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700"
            >
              {{ clientError }}
            </p>

            <ClientPanel
              :snapshot="snapshot"
              :disabled="isServer"
              @create="onCreateClient"
              @stop="onStop('client')"
            />
            <StatusLogPanel :snapshot="snapshot" :logs="logs" />
          </div>
        </Transition>
      </div>

      <!-- Settings tab -->
      <div v-else>
        <Transition name="view" appear>
          <div class="mx-auto max-w-2xl">
            <PublicNodesPanel
              :nodes="nodes"
              :disabled="busy"
              @add="add"
              @remove="remove"
              @update-node="onUpdateNode"
              @restore-default="restoreDefault"
            />
          </div>
        </Transition>
      </div>
    </main>

    <!-- Icon preview lightbox -->
    <Teleport to="body">
      <Transition name="fade">
        <div
          v-if="iconPreview"
          class="fixed inset-0 z-50 flex items-center justify-center bg-stone-900/40 p-6 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label="应用图标预览"
          @click="iconPreview = false"
        >
          <div class="ui-card p-6 text-center" @click.stop>
            <img
              src="/icon.png"
              alt="mc-network 应用图标"
              class="mx-auto h-64 w-64 rounded-2xl shadow-card"
            />
            <p class="mt-4 text-sm font-semibold text-stone-900">mc-network</p>
            <p class="mt-1 text-xs text-stone-500">Minecraft P2P 联机工具</p>
            <button type="button" class="ui-btn-secondary mt-4" @click="iconPreview = false">
              关闭
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.fade-enter-active {
  transition: opacity 0.2s ease;
}
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.view-enter-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}
.view-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.view-enter-from {
  opacity: 0;
  transform: translateX(16px);
}
.view-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}

@media (prefers-reduced-motion: reduce) {
  .view-enter-active,
  .view-leave-active {
    transition: none;
  }
  .view-enter-from,
  .view-leave-to {
    opacity: 1;
    transform: none;
  }
}
</style>
