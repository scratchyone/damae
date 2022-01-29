# Damae

Damae is a simple CLI tool for mass-deleting tweets. Its name comes from the Latin phrase "damnatio memoriae", which means "condemnation of memory".

## Installation

Clone this github repository to your local machine.

```sh
cd damae
cargo install --path .
```

## Usage

To use Damae, you must apply for a Twitter developer account, and obtain a consumer key and a consumer secret. You must also download your Twitter data archive, and extract it. Once you have all the requirements, you can run Damae with the following command:

```sh
damae [OPTIONS] <ARCHIVE_PATH> <CONSUMER_KEY> <CONSUMER_SECRET>
```

This will prompt you to open a url in your browser, authorize access to your account, and paste the resulting PIN into Damae. This allows you to use Damae to delete tweets from multiple different accounts without having to register for a developer account more than once. If you'd prefer to specify the access token and access token secret yourself, you can use the following command:

```sh
damae [OPTIONS] <ARCHIVE_PATH> <CONSUMER_KEY> <CONSUMER_SECRET> [ACCESS_TOKEN] [ACCESS_TOKEN_SECRET]
```

### Automation

To use Damae as part of an automated script, you can pass `-y` to bypass all confirmation prompts. You'll likely want to use this along with specifying an access token and access token secret, to avoid the manual authorization process.

#### Important

It is highly recommended that you run Damae with the `--dry-run` option first, to preview the changes without actually deleting anything and prevent accidentally deleting tweets. If you forget to do this, by default Damae will still prompt you for confirmation before starting the deletion process.

### Options

```
--before <OLDER_THAN>       If enabled, the tool will only delete tweets that are older than
                            the given date (in the format YYYY-MM-DD)
--dry-run                   If enabled, the tool will avoid actually executing the delete
                            operations
-h, --help                  Print help information
--max-tasks <MAX_TASKS>     Maxiumum number of concurrent deletion tasks [default: 10]
--replies-only              If enabled, the tool will only delete reply tweets
--top-level-only            If enabled, the tool will only delete top-level tweets
-V, --version               Print version information
-y, --yes                   Bypass all confirmation prompts
```
