import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "board",
      component: () => import("@/views/BoardView.vue"),
    },
    {
      path: "/inbox",
      name: "inbox",
      component: () => import("@/views/InboxView.vue"),
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
    {
      path: "/todo/:number",
      name: "todo-detail",
      component: () => import("@/views/TodoDetailView.vue"),
      props: (route) => ({ number: Number(route.params.number) }),
    },
  ],
});

export default router;
