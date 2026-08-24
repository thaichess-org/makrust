# TODO

## App level

- [ ] Set up logs and tracing for the backend.
- [ ] Research best and cheapest way to see these logs and have app observability, see best way or service to
      set up some alerts if app degrades.

## Auth & Sessions

- [ ] Sign in logic.
- [ ] Session creation logic — update `users.last_seen_at` when it is `NULL` or older than ~15 minutes.
- [ ] Sign out logic — remove user's session.
- [ ] Password reset via email.

## Email

- [ ] Send emails using Postmark.

## User

- [ ] Allow user to update profile data: bio, display name, country code.
- [ ] Resolve full country name and flag from `users.country_code` for the UI (or have the UI look this up
      itself with a local config file).
- [ ] Allow user to send and accept friend requests. Make sure you can only send one at a time to the same
      user.
- [ ] Send messages to other users.
- [ ] Allow user to block other users. Should the blocked user know they're being blocked?

## Game Logic

- [ ] Create a game seek.
- [ ] Cancel a game seek. Seeks can't be updated, user needs to create a new one.
- [ ] Create a game.
- [ ] Update a game.
- [ ] Create short game ids, make sure they're unique.
- [ ] Find a game by its short id.

## Frontend

- [ ] Set up React UI - setup linter, prettier, select library for state management, queries and css.
- [ ] Create sign in/out components.
- [ ] Work on the basic look of the player's profile. Maybe find way to create some avatar on the FE?
- [ ] Add React bindings for the makruk-ground board library.
- [ ] Deploy makrukops library to NPM.
- [ ] Find best way to trigger alerts for when user receives a message or app event. Maybe user SSE?
