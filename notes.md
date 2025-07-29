- mobile-first
- [ ] oath flow

  - [ ] login page
    - [ ] GET /login
    - [ ] POST /login (redirect to oauth)
  - [ ] GET /oauth/callback

- [ ] review page

  - [ ] eagerly load all stacks
  - [ ] change current stack
  - [ ] current card
    - [ ] front/back
      - [ ] text
      - [ ] language
        - iso 639-1 [codes-iso-639](https://crates.io/crates/codes-iso-639)
      -
    - [ ] show # remaining

- [ ] edit page

  - [ ] create stack
  - [ ] clone stack
  - [ ] delete stack
  - [ ] edit stack
    - [ ] edit label/lang
    - [ ] add card to stack
    - [ ] delete card from stack
    - [ ] edit card in stack
    - [ ] copy card to other stack

  user has-many stack
  stack has-many card

- define schema
  - events also have `uri` which we can use as pk
- filter firehose on by our collections
  - `update` and `create` events
  -

## data model

- `card`
  - `uri text primary key`
    - `at://{message.did}/{commit.collection}/{commit.rkey}`
  - `back_lang varchar(2) not null`
    - iso 639-1
  - `back_text text not null`
  - `front_lang varchar(2) not null`
    - iso 639-1
  - `front_text text not null`
  - `author_did text not null`
    - `message.did`
  - `stack_id uuid references stack(id) not null`
  - `created_at timestamp without timezone not null`
  - `indexed_at timestamp without timezone not null`
  - index on `author_did`
    - maybe on `front_lang`, `back_lang`
- `stack`
  - `id uuid primary key default gen_random_uuid()`
  - `author_did text not null`
    - `message.did`
  - `label text not null`
  - `back_lang varchar(2)`
  - `front_lang varchar(2)`
  - `created_at timestamp without timezone not null`
  - `indexed_at timestamp without timezone not null`
  - index on `author_did`
