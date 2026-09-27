<script setup>
const emit = defineEmits(["select"]);
</script>

<template>
  <div class="relative flex flex-col items-center overflow-hidden pt-16 pb-10 text-center">
    <!-- Sun glow behind hero -->
    <div
      class="pointer-events-none absolute -top-40 left-1/2 h-72 w-[46rem] -translate-x-1/2 rounded-full bg-gradient-to-b from-amber-200/60 via-orange-100/40 to-transparent blur-3xl"
      aria-hidden="true"
    />

    <div class="relative animate-fade-in-up" style="animation-delay: 0ms">
      <span
        class="inline-flex items-center gap-1.5 rounded-full border border-amber-200/80 bg-amber-50 px-3 py-1 text-xs font-medium text-amber-700"
      >
        <img src="/icon.png" alt="" class="h-4 w-4 rounded" />
        P2P 联机 · 无需公网 IP
      </span>
    </div>

    <h2
      class="relative mt-5 animate-fade-in-up text-3xl font-bold tracking-tight text-stone-900"
      style="animation-delay: 60ms"
    >
      Minecraft 联机网络
    </h2>

    <p
      class="relative mt-3 animate-fade-in-up text-sm text-stone-600"
      style="animation-delay: 120ms"
    >
      选择你想要进行的联机方式
    </p>
    <p
      class="relative mt-1 animate-fade-in-up text-xs text-stone-500"
      style="animation-delay: 160ms"
    >
      创建网络，邀请朋友一起进入 Minecraft
    </p>

    <!-- Choice cards -->
    <div class="relative mt-10 grid w-full max-w-2xl gap-4 px-0 sm:grid-cols-2">
      <!-- Host -->
      <button
        type="button"
        class="animate-fade-in-up group flex cursor-pointer flex-col items-center rounded-2xl border border-stone-200 bg-white px-6 py-8 text-center shadow-card transition-colors duration-200 hover:border-amber-300 hover:shadow-card-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-500/40"
        style="animation-delay: 220ms"
        @click="emit('select', 'server')"
      >
        <span
          class="flex h-16 w-16 items-center justify-center rounded-2xl border border-amber-200/70 bg-gradient-to-br from-amber-100 to-orange-50 text-amber-600 transition-colors duration-200 group-hover:from-amber-200/70 group-hover:to-orange-100/60"
        >
          <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 12 8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
          </svg>
        </span>
        <span class="mt-4 text-base font-semibold text-stone-900">我想当房主</span>
        <span class="mt-2 text-xs leading-relaxed text-stone-500">
          创建自己的联机网络<br />并邀请玩家
        </span>
        <svg
          class="mt-5 h-5 w-5 text-amber-500 transition-transform duration-200 group-hover:translate-x-1"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
        </svg>
      </button>

      <!-- Guest -->
      <button
        type="button"
        class="animate-fade-in-up group flex cursor-pointer flex-col items-center rounded-2xl border border-stone-200 bg-white px-6 py-8 text-center shadow-card transition-colors duration-200 hover:border-orange-300 hover:shadow-card-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-500/40"
        style="animation-delay: 320ms"
        @click="emit('select', 'client')"
      >
        <span
          class="flex h-16 w-16 items-center justify-center rounded-2xl border border-orange-200/70 bg-gradient-to-br from-orange-100 to-amber-50 text-orange-600 transition-colors duration-200 group-hover:from-orange-200/70 group-hover:to-amber-100/60"
        >
          <svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 12h.01M12 12h.01M18 12h.01M9 12a3 3 0 1 0 0 6 3 3 0 0 0 0-6Zm6 0a3 3 0 1 0 0 6 3 3 0 0 0 0-6Zm-3-6v6m-3-3h6M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12Z" />
          </svg>
        </span>
        <span class="mt-4 text-base font-semibold text-stone-900">我要当房客</span>
        <span class="mt-2 text-xs leading-relaxed text-stone-500">
          加入朋友的联机<br />网络一起游戏
        </span>
        <svg
          class="mt-5 h-5 w-5 text-orange-500 transition-transform duration-200 group-hover:translate-x-1"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3" />
        </svg>
      </button>
    </div>

    <p
      class="relative mt-12 animate-fade-in-up text-xs tracking-widest text-stone-500"
      style="animation-delay: 420ms"
    >
      安全 · 简单 · 快速连接 Minecraft
    </p>
  </div>
</template>
