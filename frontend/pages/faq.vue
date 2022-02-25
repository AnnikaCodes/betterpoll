<template>
    <main>
        <NavigationMenu current="/faq" />
        <section class="hero">
            <div class="hero-body">
                <h1 class="title">
                  Frequently Asked Questions
                </h1>

                <h2 class="title-small title">
                  What is BetterPoll?
                </h2>
                <p>
                  BetterPoll is a website that lets you quickly and easily
                  make ranked-choice polls and share them on the Internet.
                </p>

                <br>
                <h2 class="title-small title">
                  Can I suggest features or report concerns?
                </h2>
                <p>
                    Absolutely! We're always happy to hear feedback; feel free to email us at
                    <a href="mailto:support@betterpoll.cc">support@betterpoll.cc</a>.
                    <br>
                    If you've encountered a bug, you can either send email to the address above, or report it on
                    <a href="https://github.com/AnnikaCodes/betterpoll/issues/new">GitHub</a>.
                </p>

                <br>
                <h2 class="title-small title">
                  I'm a programmer. Can I contribute to BetterPoll or read the source code?
                </h2>
                <p>
                  Yes! BetterPoll is open source; you can find the source code in
                  <a href="https://github.com/AnnikaCodes/betterpoll/">our GitHub repository</a>.
                  <br>
                  The backend is written in Rust and uses PostgreSQL as a database. It uses the
                  <a href="https://rocket.rs">Rocket</a> web server and the
                  <code><a href="https://docs.rs/tallystick">tallystick</a></code> crate to handle vote tallying.

                  The frontend is written with the <a href="https://vuejs.org">Vue</a> framework
                  and hosted via Nuxt and GitHub Pages.
                </p>

                <br>
                <h2 class="title-small title">
                  Who created BetterPoll?
                </h2>
                <p>
                  BetterPoll was created by Annika
                  (<a href="https://github.com/AnnikaCodes">@AnnikaCodes</a> on GitHub).
                  Her last name and email address are not publicly available,
                  but if you have a reason to need them or just want to talk to her about BetterPoll,
                  she can be reached at <a href="mailto:support@betterpoll.cc">support@betterpoll.cc</a>.
                </p>
            </div>
        </section>
    </main>
</template>
<script lang="ts">
import Vue from 'vue'

export default Vue.extend({
  name: 'AboutPage',
  head() {
    return {
      title: 'About | BetterPoll',
    }
  },
})
</script>
