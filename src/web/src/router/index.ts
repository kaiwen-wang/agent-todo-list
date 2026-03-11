import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      redirect: "/board",
    },
    {
      path: "/inbox",
      name: "inbox",
      component: () => import("@/views/InboxView.vue"),
    },
    {
      path: "/board",
      name: "board",
      component: () => import("@/views/BoardView.vue"),
    },
    {
      path: "/list",
      name: "list",
      component: () => import("@/views/ListView.vue"),
    },
    {
      path: "/members",
      name: "members",
      component: () => import("@/views/MembersView.vue"),
    },
  ],
});

export default router;
