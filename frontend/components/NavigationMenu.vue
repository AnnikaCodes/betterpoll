<template>
    <!-- should also highlight which page you're looking at -- use something like <NavigationMenu current="/new" /> -->
    <b-navbar>
        <template #brand>
            <nuxt-link class="navbar-item" to="/">
                <img :src="require('@/static/navbar-logo.png')">
            </nuxt-link>
        </template>

        <template #start>
            <b-navbar-item tag="nuxt-link" to="/">
                <span :class="current === '/' ? 'has-text-primary' : ''">
                    Home
                </span>
            </b-navbar-item>
            <b-navbar-item tag="nuxt-link" to="/new">
                <span :class="current === '/new' ? 'has-text-primary' : ''">
                    Create
                </span>
            </b-navbar-item>
            <b-navbar-item tag="nuxt-link" to="/about">
                <span :class="current === '/about' ? 'has-text-primary' : ''">
                    About
                </span>
            </b-navbar-item>
        </template>

        <template #end>
            <b-navbar-item class="buttons">
                <PollCreateButton />
            </b-navbar-item>
        </template>
    </b-navbar>
</template>

<script>
export default {
  props: {
    current: {
      type: String,
      required: true,
    },
  },
}
</script>

