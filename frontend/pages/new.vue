// Creates a new poll by gathering information from the user and sending it to the API.
<template>
    <main>
        <NavigationMenu current="/new" />

        <section class="section">
            <h1 class="title">
                Create a new poll
            </h1>
            <form
                @submit.prevent="makePoll(
                    name,
                    description,
                    candidates.map(c => c.toString()),
                    Math.floor((endTime - Date.now()) / 1000),
                    numWinners,
                    id,
                    protection
                )"
            >
            <!--
                We use an element here because programmatically setting a loading with $buefy is
                probably possible but poorly documented :(
            -->
            <b-loading v-model="isLoading" />
            <b-field label="Title">
                <b-input
                    v-model="name"
                    type="text"
                    required
                    validation-message="Must be between 1 and 1024 characters"
                    placeholder="Give your poll a descriptive title"
                    maxlength="1024"
                    minlength="1"
                />
            </b-field>

            <b-field label="Description">
              <b-input
                v-model="description"
                type="textarea"
                maxlength="10000"
                placeholder="Give your poll a description"
                validation-message="Must be fewer than 10,000 characters"
              />
            </b-field>

            <b-field label="Choices">
                <b-taginput
                    v-model="candidates"
                    icon="label"
                    placeholder="Your poll needs at least 2 choices"
                    aria-close-label="Remove this choice"
                    maxlength="1024"
                    minlength="1"
                    maxtags="1024"
                    type="is-info"
                    @input="maxWinners = candidates.length - 1"
                />
            </b-field>

            <b-field label="Number of winners">
                <b-numberinput v-model="numWinners" :max="maxWinners" min="1" required />
            </b-field>

            <b-field label="Poll closing time">
                <b-datetimepicker
                    v-model="endTime"
                    placeholder="Date and time at which your poll will close and results will be available"
                    :min-datetime="new Date(Date.now() + 10000)"
                    required
                />
            </b-field>

            <b-field>
                <b-checkbox v-model="protection">
                    Prevent double voting via IP address <small>(This will record voters' IP adddresses.)</small>
                </b-checkbox>
            </b-field>

            <b-field label="Custom URL (optional)">
                <p class="content">
                    {{ $config.DOMAIN }}/poll/
                </p>

                <b-input
                    ref="customURL"
                    v-model="id"
                    type="text"
                    placeholder="Give your poll a custom URL"
                    maxlength="32"
                    minlength="2"
                    size="is-small"
                    @change.native="validateID"
                />
            </b-field>

              <div class="control">
                <button class="button is-link">
                    Create poll
                </button>
              </div>
            </form>
        </section>
    </main>
</template>

<script lang="ts">
import Vue from 'vue'
const ID_NORMAL_VALIDITY = 'Must be between 2 and 32 characters'

export default Vue.extend({
  name: 'IndexPage',
  data() {
    return {
      maxWinners: 0,
      candidates: [],
      numWinners: 1,
      title: null,
      tagValidity: '',
      description: '',
      protection: false,
      endTime: new Date(Math.floor(Date.now() / 60_000) * 60_000 + 24 * 60 * 60 * 1000), // A day in the future
      id: null,
      isLoading: false,
      idCustomValidity: ID_NORMAL_VALIDITY,
      idValidateIsLoading: false,
      name: '',
    }
  },
  methods: {
    async makePoll(
        name: string,
        description: string,
        candidates: string[],
        duration: number,
        numWinners: number,
        id: string | null,
        preventDoubleVoteByIP: boolean,
    ) {
      this.isLoading = true
      const json: {[k: string]: any} = {
        name,
        candidates,
        duration,
        numWinners,
        description,
        protection: preventDoubleVoteByIP ? 'ip' : 'none',
      }
      if (id) {
        json.id = id
      }

      try {
        const data = await this.$axios.$post(`${this.$config.API_URL}/create`, json)
        if (!data.success) {
          this.isLoading = false
          this.$buefy.toast.open({
            duration: 5000,
            message: data.error || 'There was an error creating the poll',
            type: 'is-danger',
          })
        } else {
          // Success!
          this.isLoading = false
          this.$router.push(`/poll/${data.id}`)
          this.$buefy.toast.open({
            duration: 5000,
            message: 'The poll was successfully created!',
            type: 'is-success',
          })
        }
      } catch (e) {
        this.isLoading = false
        this.$buefy.toast.open({
          duration: 5000,
          message: 'An error occured contacting our servers; make sure you are connected to the Internet',
          type: 'is-danger',
        })
        console.error(`An error occurred while POSTing to /create: ${e} ${JSON.stringify(e)}`)
      }
    },

    async validateID() {
      if (!this.$refs.customURL || (this.$refs.customURL as Vue).$el) return
      const elem = (this.$refs.customURL as Vue).$el.children[0] as HTMLInputElement
      if (!elem) return

      // Can optimize this with a special, simpler check-if-ID-is-used endpoint if needed
      const url = `${this.$config.API_URL}/poll/${elem.value}`

      try {
        const data = await this.$axios.$get(url)
        if (data.success) {
          elem.setCustomValidity(`A poll already exists with that URL.`)
          elem.reportValidity()
          return
        }
      } catch (e) {
        // Suppress errors here — if a user has poor internet connection,
        // delaying validation is better than showing errors.
      }
      elem.setCustomValidity('')
    },
  },
  head() {
    return {
      title: 'Create a poll | BetterPoll',
    }
  },
})
</script>
