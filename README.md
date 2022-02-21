# betterpoll
[![Backend CI](https://github.com/AnnikaCodes/betterpoll/actions/workflows/backend.yml/badge.svg)](https://github.com/AnnikaCodes/betterpoll/actions/workflows/backend.yml) [![Frontend CI](https://github.com/AnnikaCodes/betterpoll/actions/workflows/frontend.yml/badge.svg)](https://github.com/AnnikaCodes/betterpoll/actions/workflows/frontend.yml) [![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/AnnikaCodes/betterpoll/blob/main/LICENSE)

BetterPoll is a work-in-progress website to allow users to quickly and easily create and vote in ranked-choice polls.

Please note: this project is very new. There are many TODOs littered around the code, and it is not even ready to be publicly deployed in a testing phase.

## Backend
### API
BetterPoll's backend (whose source code is located in the `backend/` directory) exposes a REST-ish API:
- `POST /poll/<pollid>/vote` with candidate choices to vote
    - Provided data should be JSON of the form `{"choices":[]}`, where the `choices` key is an array of candidate strings
    - Response will be `{"success": true}` or equivalent JSON if the vote succeeds, and `{"success": false, "error": <errorstring>}` or equivalent if it fails (where `<errorstring>` is a string explaining the error that occured)
- `GET /poll/<pollid>` to get info about a poll
    - In the event of an error, the response will be JSON of the form `{"success": false, "error": <errorstring>}`, where `<errorstring>` is a human-readable string describing the error that occurred.
    - On success, the response will be JSON with the following properties:
        - `success` (boolean): `true`.
        - `name` (string): the name of the poll.
        - `description` (string): the poll's description.
        - `candidates` (array of strings): choices for which users can vote.
        - `creationTime` (integer): UNIX timestamp at which the poll began (in seconds).
        - `endingTime` (integer): UNIX timestamp at which the poll ends (in seconds).
        - `numWinners` (integer): number of winners the poll has.
        - `protection` (string or null): `"ip"` if votes by the same IP address are forbidden, and `null` otherwise.
        - `numVotes` (integer): the number of votes cast so far.
        - `ended` (boolean): `true` if the poll has ended, otherwise `false`.
    - If the poll has ended, the following additional properties will be specified in the response JSON:
        - `winners` (array of strings): the winner(s) of the poll. May be more/less than `numWinners` if multiple winners have the same rank in the overall tally.
- `POST /create` to create a poll
    - Provided data should be JSON, with the following **mandatory** properties:
        - `name` (string): the name for the poll.
        - `description` (string): a description of the poll.
        - `candidates` (array of strings): choices for which users can vote. Should be between 2 and 1024 in length.
        - `duration` (integer): the amount of time after which the poll will expire, in seconds. Must be positive.
        - `numWinners` (integer): the number of winners that the poll can have. Must be greater than 0 and less than the number of candidates provided.
    - The following properties are **optional**:
        - `id` (string): a custom URL for the poll. Must be a string composed of letters A-Z (upper or lowercase), numbers 0-9, `_`, `.` and `-`, with at least 1 and at most 32 characters.
        - `protection` (string): the protection method to use to prevent double voting. Currently, the only acceptable values are `ip` (prevents multiple votes from the same IP address) and `none` (allows all incoming votes). In the future, more protection methods may be implemented.
    - Response on success is JSON of the form `{"success": true, "id": <id>}`, where `<id>` is the poll's ID. On error, the response will be JSON of the form `{"success": false, "error": <errorstring>}`, where `<errorstring>` is a human-readable string describing the error that occurred.

### Database
BetterPoll currently uses PostgreSQL for its database, although it's possible that alternative databases will be added in the future.

Database tests can be disabled with the `no-db-test` feature.

There is a database schema at `backend/schema.sql`; you'll need to run this to set up the requisite tables before starting the backend server.

## Frontend
BetterPoll's frontend is written in Vue and located in the `frontend/` directory.

I plan to generate a static site from this that can be served through something like GitHub Pages.

## Configuration
Configure the databases in `Rocket.toml`; an [example](https://github.com/AnnikaCodes/betterpoll/blob/main/backend/Rocket.example.toml) is provided.

You'll also need to specify `ALLOWED_ORIGINS` as an environment variable (or in a `.env` file); it is a regular expression specifying allowed origins for CORS.

You may optionally specify the `API_URL` environment variable (to use an alternate backend) or the `DOMAIN` environment variable (which specifies the domain used in the UI display for custom URLs). However, this is optional; sane defaults are provided.

## Voting algorithms
[`tallystick`](https://crates.io/crate/tallystick) is used to provide implementations of the voting algorithms. Currently, only the Schulze method is supported, but it's definitely possible to add more in the future.
