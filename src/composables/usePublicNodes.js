import { ref, watch } from "vue";

const KEY = "mc-network.publicNodes";

export const DEFAULT_NODES = [
  "tcp://public.easytier.top:11010",
  "tcp://public2.easytier.cn:54321",
  "https://etnode.zkitefly.eu.org/node1",
  "https://etnode.zkitefly.eu.org/node2",
];

export function usePublicNodes() {
  const stored = localStorage.getItem(KEY);
  const nodes = ref(stored ? JSON.parse(stored) : [...DEFAULT_NODES]);

  watch(nodes, (v) => localStorage.setItem(KEY, JSON.stringify(v)), {
    deep: true,
    flush: "sync",
  });

  return {
    nodes,
    add: (uri) => nodes.value.push(uri),
    remove: (index) => nodes.value.splice(index, 1),
    restoreDefault: () => (nodes.value = [...DEFAULT_NODES]),
  };
}
