// Skapar ett nytt val!
// Denna sida behöver samla in information från användaren och skicka det till API:n.
// Jag vet inte hur man gör denna med Vue...
/*
    - Provided data should be JSON, with the following **mandatory** properties:
        - `name` (string): the name for the poll.
        - `candidates` (array of strings): choices for which users can vote. Should be between 2 and 1024 in length.
        - `duration` (integer): the amount of time after which the poll will expire, in seconds. Must be positive.
        - `numWinners` (integer): the number of winners that the poll can have. Must be greater than 0 and less than the number of candidates provided.
    - The following properties are **optional**:
        - `id` (string): a custom URL for the poll. Must be a string composed of letters A-Z (upper or lowercase), numbers 0-9, `_`, `.` and `-`, with at least 1 and at most 32 characters.
        - `protection` (string): the protection method to use to prevent double voting. Currently, the only acceptable values are `ip` (prevents multiple votes from the same IP address) and `none` (allows all incoming votes). In the future, more protection methods may be implemented.

    `protection` - checkbox
    `id` - text, optional
    `duration` - calculated by date/time picker
    `numWinners` - number from dropdown, limited by candidates
    `candidates` - list of text fields
    `name` - text field
*/

// TODO: submit to the API
// TODO: show polls (voting UI & results)
<template>
    <main>
        <NavigationMenu />

        <section class="section">
            <h1 class="title">
                Create a new poll!
            </h1>
            <b-field label="Title">
                <b-input
                    v-model="title"
                    type="text"
                    required
                    validation-message="Must be between 1 and 1024 characters"
                    placeholder="Give your poll a descriptive title"
                    maxlength="1024"
                    minlength="1"
                />
            </b-field>


            <!-- TODO: validate if the custom URL/ID is taken before submission -->
            <b-field label="Custom URL">
              <p class="content">{{ host }}/poll/</p><b-input
                    v-model="id"
                    type="text"
                    validation-message="Must be between 2 and 32 characters"
                    placeholder="Give your poll a custom URL"
                    maxlength="32"
                    minlength="2"
                    size="is-small"
                />
            </b-field>

            <b-field label="Choices">
                <!--
                    ~~TODO: validate that there aren't 0/1 tags~~
                    ehhhhh it's technically OK:
                    if there are <2 tags, you won't be to set a number of winners and thus can't submit the form
                -->
                <b-taginput
                    v-model="tags"
                    icon="label"
                    placeholder="Your poll needs at least 2 choices"
                    aria-close-label="Remove this choice"
                    maxlength="1024"
                    minlength="1"
                    maxtags="1024"
                    type="is-info"
                    @input="maxWinners = tags.length - 1"
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
                <b-checkbox v-model="nodoublevote">Prevent double voting via IP address <small>(This will record voters' IP adddresses.)</small></b-checkbox>
            </b-field>

              <div class="control">
                <button class="button is-link">Create poll</button>
            </div>
        </section>
    </main>
</template>

<script lang="ts">
import Vue from 'vue'

const host = process.browser ? window.location.host : ''

export default Vue.extend({
  name: 'IndexPage',
  data () {
    return {
      maxWinners: 0,
      tags: [],
      numWinners: 1,
      title: null,
      tagValidity: '',
      host
    }
  }
})
</script>
