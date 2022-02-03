<template>
    <!-- TODO: make this work on mobile, maybe with a Buefy thing -->
    <nav class="navbar" role="navigation" aria-label="main navigation">
        <div class="navbar-brand">
            <nuxt-link class="navbar-item" to="/">
                <img src="https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fwww.foundrygroup.com%2Fwp-content%2Fuploads%2F2018%2F02%2FPlaceholder-Logo.png&f=1&nofb=1" width="112" height="28">
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
            </div>

            <div class="navbar-end">
                <div class="navbar-item">
                    <div class="buttons">
                        <PollCreateButton />
                        <a class="button is-light" href="https://github.com/AnnikaCodes/bettervote">
                            <b-icon icon="github"></b-icon>
                            <strong>GitHub</strong>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    </nav>
</template>
