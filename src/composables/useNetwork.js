import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { readonly, ref } from "vue";

const snapshot = ref({ mode: { mode: "idle" }, forwards: [] });
const logs = ref([]);
const scan = ref({ hit: false, port: null });

export function useNetwork() {
  listen("network-status", (e) => (snapshot.value = e.payload));
  listen("log-line", (e) => {
    logs.value = [...logs.value.slice(-200), e.payload];
  });
  listen("mc-scan", (e) => (scan.value = e.payload));

  return {
    snapshot: readonly(snapshot),
    logs: readonly(logs),
    scan: readonly(scan),
    refresh: async () => {
      snapshot.value = await invoke("get_status");
      logs.value = await invoke("get_logs");
    },
  };
}

export function startServer({ name, secret, publicNodes }) {
  return invoke("create_server_network", { name, secret, publicNodes });
}

export function startClient({ name, secret, publicNodes, mcPort }) {
  return invoke("create_client_network", { name, secret, publicNodes, mcPort });
}

export function stopNetwork() {
  return invoke("stop_network", undefined);
}
