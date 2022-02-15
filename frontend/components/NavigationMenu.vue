<template>
    <!-- TODO: make this work on mobile, maybe with a Buefy thing -->
    <!-- should also highlight which page you're looking at -- use something like <NavigationMenu current="/new" /> -->
    <nav class="navbar" role="navigation" aria-label="main navigation">
        <div class="navbar-brand">
            <nuxt-link class="navbar-item" to="/">
                <!-- TODO: Make and add logo -->
                <img
                    src="https://www.foundrygroup.com/wp-content/uploads/2018/02/Placeholder-Logo.png"
                    width="112"
                    height="28"
                >
            </nuxt-link>
        </div>

        <div id="navbarBasicExample" class="navbar-menu">
            <div class="navbar-start">
                <nuxt-link class="navbar-item" to="/">
                    Home
                </nuxt-link>

                <nuxt-link class="navbar-item" to="/new">
                    Create
                </nuxt-link>

                <nuxt-link class="navbar-item" to="/about">
                    About
                </nuxt-link>

                <nuxt-link class="navbar-item" to="/contact">
                    Contact
                </nuxt-link>
            </div>

            <div class="navbar-end">
                <div class="navbar-item">
                    <div class="buttons">
                        <PollCreateButton />
                        <a class="button is-primary" href="https://github.com/AnnikaCodes/bettervote">
                            <b-icon icon="github" /><strong>GitHub</strong>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </nav>
</template>
