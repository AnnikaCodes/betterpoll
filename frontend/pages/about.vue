<template>
    <main>
        <NavigationMenu current="/about" />
        <section class="hero is-info">
            <div class="hero-body">
                <h1 class="title">
About BetterPoll
</h1>

                <p>
                    BetterPoll is an app that lets you make quick, easy, working ranked-choice polls.
                    It's open source, meaning you can read the
                    <a href="https://github.com/AnnikaCodes/betterpoll" class="has-text-black">source code</a> yourself.
                </p>

                <p>
                    If you have questions, concerns, or suggestions, feel free to email us at
                    <a href="mailto:betterpoll.cc@gmail.com" class="has-text-black">betterpoll.cc@gmail.com</a>.

                    We welcome your feedback!
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
