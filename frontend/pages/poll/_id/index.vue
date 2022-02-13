// View/vote in a poll
<template>
    <main>
        <NavigationMenu />
        <b-loading v-model="isLoading" />
        <section class="section">
          <h1 class="title">
              Poll: {{ name }}
          </h1>


          <div v-if="ended" id="expired-poll">
            <section class="section hero is-danger">
              <h1 class="title">
                Poll expired
              </h1>

              <p>
                Viewing expired polls is not currently supported.
                <!-- TODO: implement -->
              </p>
            </section>
          </div>

          <div id="ongoing-poll" v-else>
            <p>
            </p>
            <p>
            </p>

            <b-message
              v-if="isIPOnly"
              type="is-info"
              aria-close-label="Close message">
              This poll was created on
              {{ creationTime.toLocaleString(undefined, { dateStyle: 'full', timeStyle: 'short' }) }};
              it will expire on {{ endTime.toLocaleString(undefined, { dateStyle: 'full', timeStyle: 'short' }) }}.
              <br />
              This poll will ultimately have <strong>{{ numWinners }}</strong> winner{{ numWinners === 1 ? '' : ' ' }}.
              <br />
              {{ numVotes }} vote{{ numVotes === 1 ? '' : 's' }} have been cast in this poll so far.
              <br />
              <strong>Your IP address will be recorded when you vote in this poll; it will be used to prevent double voting.</strong>
            </b-message>

          <h2 class="title" style="font-size:1.5rem;">Rank your choices</h2>
          <!-- TODO: add info question mark here -->
          <table class="table is-striped is-hoverable is-fullwidth">
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
          </div>
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
      this.endTime = new Date(data.endingTime * 1000)
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
