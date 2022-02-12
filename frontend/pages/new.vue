// Skapar ett nytt val!
// Denna sida behöver samla in information från användaren och skicka det till API:n.
// Jag vet inte hur man gör denna med Vue...

// TODO: show polls (voting UI & results)
<template>
    <main>
        <NavigationMenu />

        <section class="section">
            <h1 class="title">
                Create a new poll
            </h1>
            <form
                @submit.prevent="makePoll(
                    name,
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

            <b-field label="Choices">
                <!--
                    ~~TODO: validate that there aren't 0/1 tags~~
                    ehhhhh it's technically OK:
                    if there are <2 tags, you won't be to set a number of winners and thus can't submit the form
                -->
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

            <!-- TODO: validate if the custom URL/ID is taken before submission -->
            <b-field label="Custom URL (optional)">
                <p class="content">
                    {{ domain }}/poll/
                </p>

                <b-input
                    v-model="id"
                    type="text"
                    validation-message="Must be between 2 and 32 characters"
                    placeholder="Give your poll a custom URL"
                    maxlength="32"
                    minlength="2"
                    size="is-small"
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
import { BETTERVOTE_API_URL, DOMAIN } from '../constants'

export default Vue.extend({
  name: 'IndexPage',
  data() {
    return {
      maxWinners: 0,
      candidates: [],
      numWinners: 1,
      title: null,
      tagValidity: '',
      protection: false,
      endTime: new Date(Math.floor(Date.now() / 60_000) * 60_000 + 24 * 60 * 60 * 1000), // A day in the future
      id: null,
      domain: `${DOMAIN}`,
      isLoading: false,
      name: '',
    }
  },
  methods: {
    async makePoll(
        name: string,
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
        protection: preventDoubleVoteByIP ? 'ip' : 'none',
      }
      if (id) {
        json.id = id
      }

      try {
        const data = await this.$axios.$post(`${BETTERVOTE_API_URL}/create`, json)
        if (!data.success) {
          // TODO: handle error, with a modal/data.error
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
  },
})
</script>
