<template>
    <!-- should also highlight which page you're looking at -- use something like <NavigationMenu current="/new" /> -->
    <b-navbar>
        <template #brand>
            <nuxt-link class="navbar-item" to="/">
                <!-- TODO: Make and add logo once we know the domain name -->
                <img
                    src="https://www.foundrygroup.com/wp-content/uploads/2018/02/Placeholder-Logo.png"
                    width="112"
                    height="28"
                >
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
            <b-navbar-item tag="nuxt-link" to="/contact">
                <span :class="current === '/contact' ? 'has-text-primary' : ''">
                    Contact
                </span>
            </b-navbar-item>
        </template>

        <template #end>
            <b-navbar-item class="buttons">
                <PollCreateButton />
                <a class="button is-primary" href="https://github.com/AnnikaCodes/bettervote">
                    <b-icon icon="github" /><strong>GitHub</strong>
                </a>
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

