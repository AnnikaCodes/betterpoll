# bettervote
## API
Bettervote exposes a RESTful API:
- `POST /poll/<pollid>/vote` with candidate choices to vote
    - Provided data should be JSON of the form `{"choices":[]}`, where the `choices` key is an array of candidate strings
    - Response will be `{"success": true}` or equivalent JSON if the vote succeeds, and `{"success": false, "error": <errorstring>}` or equivalent if it fails (where `<errorstring>` is a string explaining the error that occured)
- `GET /poll/<pollid>` to get info about a poll
    - In the event of an error, the response will be JSON of the form `{"success": false, "error": <errorstring>}`, where `<errorstring>` is a human-readable string describing the error that occurred.
    - On success, the response will be JSON with the following properties:
        - `success` (boolean): `true`.
        - `name` (string): the name of the poll.
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
        - `candidates` (array of strings): choices for which users can vote. Should be between 2 and 1024 in length.
        - `duration` (integer): the amount of time after which the poll will expire, in seconds. Must be positive.
        - `numWinners` (integer): the number of winners that the poll can have. Must be greater than 0 and less than the number of candidates provided.
    - The following properties are **optional**:
        - `id` (string): a custom URL for the poll. Must be a string composed of letters A-Z (upper or lowercase), numbers 0-9, `_`, `.` and `-`, with at least 1 and at most 32 characters.
        - `protection` (string): the protection method to use to prevent double voting. Currently, the only acceptable values are `ip` (prevents multiple votes from the same IP address) and `none` (allows all incoming votes). In the future, more protection methods may be implemented.
    - Response on success is JSON of the form `{"success": true, "id": <id>}`, where `<id>` is the poll's ID. On error, the response will be JSON of the form `{"success": false, "error": <errorstring>}`, where `<errorstring>` is a human-readable string describing the error that occurred.


A frontend will also need to be worked out.
That can probably go into a separate repository, but should access this API from the browser
and have a pretty interface.

TODO: look into different frontend frameworks (Vue? Svelte? Flutter?).

## Database
Implements a way to store information needed for the website.

- Postgres abstraction
    - should make it easy to switch between databases in case we need to switch to AWS/MySQL/etc
    - should store individual polls (creation time, expiry time, candidates, voting system)
- should store votes associated with a given poll (candidate choices, some way to identify the voter)
    - consider making the voter identification generic somehow???
        - may be IP address/fingerprint/both/nothing

## Configuration
Configure the databases in `Rocket.toml`; an example is provided (TODO: link here).

## Voting algorithms
Use [`tallystick`](https://crates.io/crate/tallystick) to get the Schulze method etc.