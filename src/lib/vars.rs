pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub const HELP_EXTRA_TEXT: &str = "Find the documentation website at https://fbtsecurity.fbtheaven.com/\nRun the About command to find out more (/about)";

// TODO: change this list to your own bot admin user IDs

// You need to increase the number in [u64; X] so rust knows the limit of the array
pub const BOT_ADMINS: [u64; 6] = [
    212_132_817_017_110_528,
    288_186_677_967_585_280,
    211_027_317_068_136_448,
    383_507_911_160_233_985,
    168_600_506_233_651_201,
    231_482_341_921_521_664,
]; // Azuki, Komi, Xeno, Mojo, Ellie, Wundie

// TODO: you can mass replace the name of this variable easily
// TODO: change to your own guild ID

pub const FBT_GUILD_ID: u64 = 737_168_134_502_350_849; // FBT's guild ID

// TODO: this is the channel wehre the feedback command sends it's response for you to read
pub const FEEDBACK_CHANNEL_ID: u64 = 925_599_477_283_311_636;

//pub const FBT_GUILD_ID: u64 = 838658675916275722; // My test server ID

// TODO: you need your own Redis DB, this is where you put in the login details and adress of the DB
// format: "redis://USERNAME:PASSWORD@ADDRESS:PORT/DB_INDEX"

#[cfg(feature = "database")]
pub const REDIS_ADDR: &str =
    "redis://:ForSureARealRedisPassword@google.com:6379/0";

// TODO: change to your own Meilisearch address
#[cfg(feature = "database")]
pub const MEILISEARCH_HOST: &str = "http://google.com:7777";

// TODO: change to your own Meilisearch API key
#[cfg(feature = "database")]
pub const MEILISEARCH_API_KEY: &str = "why-so-strange";

// TODO: change to your own bot token
pub const BOT_TOKEN: &str =
    "not touching this <3";

//TODO: these are popular discord bots, used to ignore their messages and stuff
// Part of blacklist for now but I should add it as a check to the excel command too
#[cfg(feature = "database")]
pub const BOT_IDS: [u64; 22] = [
    134_133_271_750_639_616,
    155_149_108_183_695_360,
    159_985_870_458_322_944,
    159_985_870_458_322_944,
    184_405_311_681_986_560,
    204_255_221_017_214_977,
    216_437_513_709_944_832,
    235_088_799_074_484_224,
    235_148_962_103_951_360,
    294_882_584_201_003_009,
    351_227_880_153_546_754,
    375_805_687_529_209_857,
    537_429_661_139_861_504,
    550_613_223_733_329_920,
    559_426_966_151_757_824,
    583_995_825_269_768_211,
    625_588_618_525_802_507,
    649_535_344_236_167_212,
    743_269_383_438_073_856,
    743_269_383_438_073_856,
    887_914_294_988_140_565,
    935_372_708_089_315_369,
];

// TODO: this is for the ticket system, change to your own ticket category ID.
// it creates new threads in TICKET_CATEGORY and moves them to CLOSED_TICKET_CATEGORY once closed
pub const TICKET_CATEGORY: u64 = 982_769_870_259_240_981;
pub const CLOSED_TICKET_CATEGORY: u64 = 983_228_142_107_918_336;

#[cfg(feature = "database")]
#[derive(Debug, poise::ChoiceParameter)]
pub enum BlacklistOutput {
    #[name = "Chat - Output resulting @, ID and Reasons to chat"]
    Chat,
    #[name = "Compact Chat - Only send resulting @ and IDs"]
    CompactChat,
    #[name = "CSV - Output all relevant info as a single .csv file"]
    Csv,
    #[name = "Json - Output all relevant info as a single .json file"]
    Json,
}
