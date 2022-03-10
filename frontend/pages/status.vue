<template>
    <main>
        <NavigationMenu current="/status" />
        <section class="hero">
            <div class="hero-body">
              <h1 class="title">
                Status
              </h1>

              <p class="container notification">
                This page provides information about the status of various components of BetterPoll.
                <br>
                If one or more of the below boxes displays an
                <b-icon icon="close-circle" class="has-text-danger" />
                icon, there is likely an outage that will impact your use of the website.
              </p>

              <StatusBox
                label="Frontend"
                :working="true"
                details="The text, menus, and other static details of the website"
              />

              <StatusBox
                label="API"
                :working="apiReachable"
                :loading="loading"
                :info="apiError"
                details="The server that handles all data, like creating or viewing a poll"
              />

              <StatusBox
                label="Database"
                :working="apiSuccess"
                :loading="loading"
                details="The place where all data about polls and votes is stored"
              />

              <StatusBox
                label="Active Polls"
                :working="activePolls"
                :loading="loading"
                details="The number of polls currently accepting votes"
              />

              <StatusBox
                label="Total Polls"
                :working="totalPolls"
                :loading="loading"
                details="The total number of polls, including those that have ended"
              />
            </div>
        </section>
    </main>
</template>
<script lang="ts">
import Vue from 'vue'

export default Vue.extend({
  name: 'StatusPage',
  data() {
    return {
      loading: true,
      apiReachable: false,
      apiSuccess: false,
      apiError: 'No issues detected!',
      totalPolls: false,
      activePolls: false,
    }
  },
  head() {
    return {
      title: 'Status | BetterPoll',
    }
  },
  async mounted() {
    try {
      const data = await this.$axios.$get(`${this.$config.API_URL}/status`)
      this.apiReachable = true

      if (data.total === null || data.active === null) {
        this.apiSuccess = false
      } else {
        this.apiSuccess = true
        this.totalPolls = data.total.toString()
        this.activePolls = data.active.toString()
      }

      this.loading = false
    } catch (e) {
      if (e.response && e.response.status === 429) {
        // Rate limiting!
        this.$buefy.toast.open({
          duration: 5000,
          message: 'You are making too many requests. Please wait a bit before trying again.',
          type: 'is-danger',
        })
      } else {
        this.apiReachable = false
        this.apiError = 'Either your Internet connection or our servers are experiencing errors'
        this.loading = false
      }
    }
  },
})
</script>
