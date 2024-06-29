# FBT Security - Rust

This is the source code for the [FBTHeaven discord bot](https://fbtsecurity.fbtheaven.com/).

[FBT Links](https://linktr.ee/FBT_Heaven)

## about

I am the developer for version 2.0, it has been a stale unmaintained project for months but I didn't want my source to just wither untouched so I removed the API keys and added a hand full of `// TODO:`s to the code for you to find what discord IDs you need to change and what other keys you need to provide.

You will need a [Redis DB](https://redis.io/) for a bunch of features, no guide on setting one up atm (or possibly every, we'll see how I feel later)
You can also update a [Meilisearch DB](https://www.meilisearch.com/) with the same data, that one command is easy to comment out if you don't want to use that too

## redis layout

The db is split into 8 "folders".
Redis is a key:value DB meaning you have just the name of the DB entry and then it's value, an example of this is the entry `user:0000000000000000000` which is an entry in the user "folder".

here are the "folders" and their descriptions:

- `authed-server-users:<DiscordServerID>`
  - This is a Redis [SET](https://redis.io/docs/latest/develop/data-types/sets/) of discord user IDs who are authenticated in the server in the DB entry
- `cleared-suer:<DiscordUserID>`
  - this is a [JSON](https://redis.io/docs/latest/develop/data-types/json/) entry of users who are cleared as okay in the DB after being flagged
  - Json format:

        ```JSON
        {
            "user_id": "0000000000000000000",
            "username": "TestUsername#0001",
            "where_found": "Name of guild",
            "reason": "Admin enters custom reason here"
        }
        ```

- `feedback:<timestamp>-<DiscordUserID>-<DiscordUserName>`
  - This is just a [String](https://redis.io/docs/latest/develop/data-types/strings/) containing whatever feedback they put in the feedback command
- `guild-settings:<DiscordGuildID>`
  - This is a [JSON](https://redis.io/docs/latest/develop/data-types/json/) entry containing if the server has auto kick enabled or not as well as the channel id for bot announcements

    ```json
    {
        "channel_id": "0000000000000000000",
        "kick": true,
        "server_name": "Name of the guild"
    }
    ```

- `monitroted-guild:<DiscordGuildID>`
  - This is a [JSON](https://redis.io/docs/latest/develop/data-types/json/) entry containing info about tracked servers. this is only inside of the `_deprecated.rs` as it was a hold over from the old Python verion's SQLite DB. more info about this one will come with the Python source code later™️

    ```json
    {
        "guild_name": "Example guild name",
        "guild_id": "0000000000000000000",
        "invite_link": "https://discord.gg/example",
        "updated": "Never",
        "status": "Unknown"
    }
    ```

- `status:commands-executed`
  - this is a simple [String](https://redis.io/docs/latest/develop/data-types/strings/) entry to track how many commands have run since the feature was implemented, it appears at the bottom of the `/about` command
- `user:<DiscordUserID>`
  - This is the largest list of entries in the DB.
  - These are [JSON](https://redis.io/docs/latest/develop/data-types/json/) entries for users who are uploaded via scrapped discords, more info on that in the next section

    ```json
    {
        "vrc_id": "ID/Profile URL enetered on a per user basis",
        "username": "DiscordUsername",
        "discord_id": "0000000000000000000",
        "offences": [
            {
                "guild_id": "0000000000000000000",
                "reason": "Reason for adding to list. typically name of discord server found, date and a key from the /key command",
                "image": "URL(s) to image(s) if user was added manually for evidence",
                "extra": "Other text here, mainly for links to extra files"
            }
        ]
    }
    ```

- `kick-whitelist`
  - This is a [SET](https://redis.io/docs/latest/develop/data-types/sets/) that I didn't put in a folder for some reason. It holds a list of DiscordUserIDs that should be ignored when running the `/excel` command.

# scrape discords

So the main use of this bot origionally was as follows:

- scrape discord using [DiscordChatExporter](https://github.com/Tyrrrz/DiscordChatExporter)
- run resulting .csv files though [my old parser](https://github.com/BuyMyMojo/discord-chat-parser)
- resulting file to the bot using the `/excel` command

This was to be used to scrape servers of bad actors such as VRC model rippers and leakers, it would add every user found inside the server into the DB with a description of the server that they are found in.

Once in the DB whenever the user joined a server with the FBT bot active the server owner/mods would get notified of a bad actor joining or optionally it would auto kick these accounts.

# that's all folks

I might clean this up further or add a branch for the old python version in the future.

good luck whoever may look upon my first large Rust project

<p style="color: rgba(0, 0, 0, 0)">
There is no database files or user entries here, look somewhere else <3
</p>
