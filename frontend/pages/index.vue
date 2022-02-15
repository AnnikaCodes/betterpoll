<template>
  <main>
    <NavigationMenu current="/" />
    <section class="hero is-info">
      <div class="hero-body">
        <p class="title">
          Simple, easy ranked-choice voting
        </p>
        <!-- eslint-disable max-len -->
        <p>
          BetterVote is a simple way to create and share ranked-choice polls.
        </p>

        <p>
          You might think that voting is simple: everyone gets one vote, and whoever has the most votes wins.
          However, this simple system has some major flaws, namely that it encourages people to vote for options that are likely to popular among others, rather than just the one they like most.
          This is called <a href="https://en.wikipedia.org/wiki/First-past-the-post_voting#Tactical_voting" class="has-text-black">tactical voting</a>, and ranked-choice voting largely prevents this.
        </p>

        <br>
        <p>
          BetterVote currently uses the <a href="https://en.wikipedia.org/wiki/Schulze_method" class="has-text-black">Schulze method</a> of ranked-choice voting, but other methods may be added in the future!
        </p>
        <!-- eslint-enable max-len -->
      </div>
    </section>
  </main>
</template>

<script lang="ts">
import Vue from 'vue'

export default Vue.extend({
  name: 'IndexPage',
})
</script>
