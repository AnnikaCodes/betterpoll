// View/vote in a poll
<template>
    <main>
        <NavigationMenu />
        <b-loading v-model="isLoading" />
        <section class="section">
          <h1 class="title">
              Poll: {{ name }}
          </h1>

        <table class="table">
          <thead>
            <tr>
              <th scope="col">Rank</th>
              <th scope="col">Choice</th>
              <th /> <!-- Removal button -->
            </tr>
          </thead>
            <draggable v-model="candidates" group="people" @start="drag=true" @end="drag=false" tag="tbody">
              <tr v-for="(choice, index) in candidates" :key="choice">
                <td>#{{ index + 1 }}</td>
                <td>{{ choice }}</td>
                <td>
                  <b-button
                    @click="candidates = candidates.filter(x => x !== choice)"
                    class="is-danger"
                    icon-left="delete"
                    type="is-small"
                  >
                    <small>Don't vote for this candidate</small>
                  </b-button>
                </td>
              </tr>
            </draggable>
        </table>
        </section>
    </main>
</template>

<script lang="ts">
import Vue from 'vue'
import draggable from 'vuedraggable'
import { BETTERVOTE_API_URL, DOMAIN } from '../../../constants'

export default Vue.extend({
  name: 'IndexPage',
  data() {
    return {
      name: '',
      candidates: [],
      creationTime: null as Date | null,
      endTime: null as Date | null,
      numWinners: 0,
      isIPOnly: false,
      numVotes: 0,
      ended: false,
      isLoading: true,
      drag: false,
    }
  },
  async mounted() {
    const id = this.$route.params.id
    try {
      const data = await this.$axios.$get(`${BETTERVOTE_API_URL}/poll/${id}`)
      if (!data.success) {
        if (!data.error) throw new Error(`no error from server`)
        this.$buefy.toast.open({
          duration: 5000,
          message: data.error,
          type: 'is-danger',
        })
        this.$router.push('/')
      }
      this.name = data.name
      this.candidates = data.candidates
      this.creationTime = new Date(data.creationTime * 1000)
      this.endTime = new Date(data.endTime * 1000)
      this.numWinners = data.numWinners
      this.isIPOnly = data.protection === 'ip'
      this.numVotes = data.numVotes
      this.ended = data.ended
      this.isLoading = false
    } catch (e) {
      this.$buefy.toast.open({
        duration: 5000,
        message: 'There was an error connecting to the server',
        type: 'is-danger',
      })
      console.error(`An error occurred GETing /poll/${id}: ${e} ${JSON.stringify(e)}`)
    }
  },
  components: {
      draggable,
  }
})
</script>
