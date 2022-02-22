// View/vote in a poll
<template>
    <main>
        <NavigationMenu current="none" />
        <b-loading v-model="isLoading" />
        <section class="section">
          <h1 class="title">
            Poll: {{ name }}
          </h1>

          <div class="container notification">
            {{ description }}
          </div>

          <div v-if="ended" id="expired-poll">
            <b-message
              type="is-info"
              aria-close-label="Close message"
            >
              <strong>This poll has ended.</strong>
              <br>
              This poll was created on
              {{ creationTime.toLocaleString(undefined, { dateStyle: 'full', timeStyle: 'short' }) }}
              and ended on {{ endTime.toLocaleString(undefined, { dateStyle: 'full', timeStyle: 'short' }) }}.
              <br>
              {{ numVotes }} vote{{ numVotes === 1 ? '' : 's' }} were cast in this poll.
            </b-message>

            <div v-if="winners.length === 0" id="no-winners">
              There were no winners.
            </div>
            <div v-else>
              <!-- TODO: move CSS to a .css file -->
              <h2 id="winners-heading" class="title">
                Winner{{ winners.length > 1 ? 's' : '' }}: <span v-for="(winner, index) in winners" :key="winner">
                  <b>{{ winner }}</b>{{ ((index === winners.length - 1) || (winners.length === 2)) ? '' : ', ' }}{{
                    index === winners.length - 2 ? ' and ' : ''
                  }}
                </span>
              </h2>
            </div>
          </div>

          <div v-else id="ongoing-poll">
            <b-message
              v-if="exists"
              type="is-info"
              aria-close-label="Close message"
            >
              This poll was created on
              {{ creationTime.toLocaleString(undefined, { dateStyle: 'full', timeStyle: 'short' }) }};
              it will end on {{ endTime.toLocaleString(undefined, { dateStyle: 'full', timeStyle: 'short' }) }}.
              <br>
              This poll will ultimately have <strong>{{ numWinners }}</strong>
              winner{{ numWinners === 1 ? '' : 's' }}.
              <br>
              {{ numVotes }} vote{{ numVotes === 1 ? ' has' : 's have' }} been cast in this poll so far.
              <br>
              <strong v-if="isIPOnly">
                Your IP address will be recorded when you vote in this poll; it will be used to prevent double voting.
              </strong>
            </b-message>
            <b-message v-else type="is-info">
              This poll does not exist.
            </b-message>


            <h2 v-if="exists" class="title title-small">
              Rank your choices
              <b-tooltip
                multilined
                type="is-primary"
                label="
                  Drag and drop the choices into your preferred order.
                  If you don't want to vote for a choice at all, click the red button to remove it.
                "
              >
                <b-icon icon="help-circle-outline" />
              </b-tooltip>
            </h2>

            <div class="flex">
              <table class="table is-hoverable">
                <thead>
                  <tr>
                    <th scope="col">
                      Rank
                    </th>
                  </tr>
                </thead>
                    <tr v-for="(c, index) in candidates" :key="index">
                      <td>#{{ index + 1 }}</td>
                    </tr>
              </table>
              <table class="table is-hoverable is-fullwidth">
                <thead>
                  <tr>
                    <th scope="col">
                      Choice
                    </th>
                    <th /> <!-- Removal button -->
                  </tr>
                </thead>
                  <draggable v-model="candidates" group="people" tag="tbody" @start="drag=true" @end="drag=false">
                    <tr v-for="choice in candidates" :key="choice">
                      <td>{{ choice }}</td>
                      <td>
                        <b-button
                          id="small-button"
                          class="is-danger"
                          icon-left="delete"
                          @click="candidates = candidates.filter(x => x !== choice)"
                        />
                      </td>
                    </tr>
                  </draggable>
              </table>
            </div>

            <div class="control">
              <button class="button is-link" @click="submit">
                  Vote
              </button>
            </div>
          </div>
        </section>
    </main>
</template>

<script lang="ts">
import Vue from 'vue'
import draggable from 'vuedraggable'

export default Vue.extend({
  name: 'IndexPage',
  components: {
    draggable,
  },
  data() {
    return {
      name: '',
      description: '',
      candidates: [],
      creationTime: new Date(0),
      endTime: new Date(0),
      numWinners: 0,
      isIPOnly: false,
      numVotes: 0,
      ended: false,
      winners: undefined as string[] | undefined,
      isLoading: true,
      drag: false,
      exists: false,
    }
  },
  async mounted() {
    const id = this.$route.params.id
    try {
      const data = await this.$axios.$get(`${this.$config.API_URL}/poll/${id}`)
      if (!data.success) {
        if (!data.error) throw new Error(`no error from server`)
        this.$buefy.toast.open({
          duration: 5000,
          message: data.error,
          type: 'is-danger',
        })
        this.isLoading = false
        return
      }
      this.name = data.name
      window.document.title = `${data.name} | ${window.document.title}`
      this.description = data.description
      this.candidates = data.candidates
      this.creationTime = new Date(data.creationTime * 1000)
      this.endTime = new Date(data.endingTime * 1000)
      this.numWinners = data.numWinners
      this.winners = data.winners
      this.isIPOnly = data.protection === 'ip'
      this.numVotes = data.numVotes
      this.ended = data.ended
      this.exists = true
      this.isLoading = false
    } catch (e) {
      let message = 'An error occured contacting our servers; make sure you are connected to the Internet'
      if (e.response.status === 429) {
        // Rate limiting!
        message = 'You are making too many requests. Please wait a bit before trying again.'
      }
      this.$buefy.toast.open({
        duration: 5000,
        message,
        type: 'is-danger',
      })
      console.error(`An error occurred GETing /poll/${id}: ${e} ${JSON.stringify(e)}`)
    }
  },
  methods: {
    async submit() {
      const id = this.$route.params.id
      if (!this.candidates.length) {
        this.$buefy.toast.open({
          duration: 5000,
          message: 'You must select at least one candidate!',
          type: 'is-danger',
        })
        this.$router.push(`/poll/${id}`)
      }
      this.isLoading = true

      try {
        const data = await this.$axios.$post(`${this.$config.API_URL}/poll/${id}/vote`, {
          choices: this.candidates,
        })

        if (!data.success) {
          this.isLoading = false
          this.$buefy.toast.open({
            duration: 5000,
            message: data.error || 'There was an error voting in the poll',
            type: 'is-danger',
          })
        } else {
          // Success!
          this.isLoading = false
          this.$buefy.toast.open({
            duration: 5000,
            message: 'You have voted successfully!',
            type: 'is-success',
          })
          this.$forceUpdate()
        }
      } catch (e) {
        this.isLoading = false
        let message = 'An error occured contacting our servers; make sure you are connected to the Internet'
        if (e.response.status === 429) {
          // Rate limiting!
          message = 'You are making too many requests. Please wait a bit before trying again.'
        }
        this.$buefy.toast.open({
          duration: 5000,
          message,
          type: 'is-danger',
        })
        console.error(`An error occurred while POSTing to /poll/${id}/vote: ${e} ${JSON.stringify(e)}`)
      }
    },
  },
})
</script>
