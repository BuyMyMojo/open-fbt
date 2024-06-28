# FBT Security - Rust

This is the source code for the FBTHeaven discord bot.

I am the developer for version 2.0, it has been a stale unmaintained project for months but I didn't want my source to just wither untouched so I removed the API keys and added a hand full of `// TODO:`s to the code for you to find what discord IDs you need to change and what other keys you need to provide.

You will need a [Redis DB](https://redis.io/) for a bunch of features, no guide on setting one up atm (or possibly every, we'll see how I feel later)
You can also update a [Meilisearch DB](https://www.meilisearch.com/) with the same data, that one command is easy to comment out if you don't want to use that too

I might clean this up further or add a branch for the old python version in the future.

good luck whoever may look upon my first large Rust project