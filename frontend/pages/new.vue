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

<template>
    <main>
        <NavigationMenu />

<script>tags=[]</script>
        <section class="section">
            <b-field label="Title">
                <b-input type="text" required validation-message="Must be between 1 and 1024 characters" maxlength="1024" minlength="1"/>
            </b-field>

            <b-field label="Choices">
                <!-- TODO: validate that there aren't 0 tags -->
                <b-taginput
                    v-model="tags"
                    v-on:input="maxWinners = Math.max(tags.length - 1, 1)"
                    icon="label"
                    placeholder="Add some choices for your poll"
                    aria-close-label="Remove this choice"
                    maxlength="1024"
                    minlength="1"
                    maxtags="1024"
                    type="is-info"
                />
            </b-field>

            <b-field label="Number of winners">
                <b-numberinput placeholder="1" :max="maxWinners" min="1" />
            </b-field>
        </section>
    </main>
</template>

<script lang="ts">
    import Vue from 'vue'

    export default Vue.extend({
        name: 'IndexPage',
        data: function() {
            return {
                maxWinners: 0,
                tags: [],
                console,
            };
        },
    });
</script>
