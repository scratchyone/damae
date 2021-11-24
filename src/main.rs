use chrono::TimeZone;
use chrono::{DateTime, NaiveDate, NaiveTime};
use clap::Parser;
use colour::*;
use egg_mode::{self, auth::verify_tokens};
use futures::StreamExt;
use indicatif::{self, ProgressBar};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
struct WrappedTweet {
    tweet: Tweet,
}
#[derive(Deserialize, Debug)]
struct Tweet {
    id: String,
    in_reply_to_status_id: Option<String>,
    created_at: String,
}

/// Damae is a tool for erasing all tweets from a twitter account.
#[derive(Parser, Clone)]
#[clap(version = "1.0", author = "Rachel")]
struct Opts {
    /// Path to the unzipped twitter archive
    archive_path: String,
    /// Consumer key for the twitter API
    consumer_key: String,
    /// Consumer secret for the twitter API
    consumer_secret: String,
    /// Access token for the twitter API
    access_token: String,
    /// Access token secret for the twitter API
    access_token_secret: String,
    /// If enabled, the tool will avoid actually executing the delete operations
    #[clap(long = "dry-run")]
    dry_run: bool,
    /// If enabled, the tool will only delete reply tweets
    #[clap(long = "replies-only")]
    replies_only: bool,
    /// If enabled, the tool will only delete top-level tweets
    #[clap(long = "top-level-only")]
    top_level_only: bool,
    /// If enabled, the tool will only delete tweets that are older than the given date
    /// (in the format YYYY-MM-DD)
    #[clap(long = "before")]
    older_than: Option<NaiveDate>,
    /// Maxiumum number of concurrent deletion tasks
    #[clap(long = "max-tasks", default_value = "10")]
    max_tasks: usize,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let tweets_path = PathBuf::from(&opts.archive_path).join("data/tweet.js");
    let tweets_str = std::fs::read_to_string(&tweets_path).unwrap();
    let tweets_str = tweets_str
        .strip_prefix("window.YTD.tweet.part0 = ")
        .unwrap();
    let mut tweets: Vec<WrappedTweet> = serde_json::from_str(tweets_str).unwrap();

    let con_token = egg_mode::KeyPair::new(opts.consumer_key.clone(), opts.consumer_secret.clone());
    let access_token =
        egg_mode::KeyPair::new(opts.access_token.clone(), opts.access_token_secret.clone());
    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: access_token,
    };

    match verify_tokens(&token).await {
        Ok(_) => green_ln!("🔓 Logged in successfully"),
        Err(e) => {
            red_ln!("🚨 {}", e);
            std::process::exit(1);
        }
    }

    if opts.replies_only {
        tweets.retain(|t| t.tweet.in_reply_to_status_id.is_some());
    }

    if opts.top_level_only {
        tweets.retain(|t| t.tweet.in_reply_to_status_id.is_none());
    }

    if let Some(older_than) = opts.older_than {
        tweets.retain(|t| {
            DateTime::parse_from_str(&t.tweet.created_at, "%a %b %d %H:%M:%S %z %Y").unwrap()
                < chrono::Utc.from_utc_datetime(&older_than.and_time(NaiveTime::from_hms(0, 0, 0)))
        });
    }

    if opts.dry_run {
        yellow_ln!("🥸 Running in dry-run mode");
    }

    green_ln!("🔎 Loaded {} tweets from archive", tweets.len());
    cyan_ln!("✨ Starting tweet deletion");

    let pb = Arc::new(Mutex::new(ProgressBar::new(tweets.len() as u64)));
    let failed_tweets = Arc::new(Mutex::new(0));
    let deleted_tweets = Arc::new(Mutex::new(0));
    let tasks = futures::stream::iter(tweets.iter().map(|tweet| {
        let failed_tweets = failed_tweets.clone();
        let deleted_tweets = deleted_tweets.clone();
        let pb = pb.clone();
        let opts = opts.clone();
        let token = token.clone();
        async move {
            let id = tweet.tweet.id.clone();
            let id = id.parse::<u64>().unwrap();
            if !opts.dry_run {
                match egg_mode::tweet::delete(id, &token).await {
                    Ok(_) => {
                        *deleted_tweets.lock().await += 1;
                    }
                    Err(e) => {
                        match e {
                            egg_mode::error::Error::TwitterError(_, te) => {
                                if te.errors.iter().any(|ec| ec.code == 144) {
                                    // Tweet already deleted
                                    *deleted_tweets.lock().await += 1;
                                } else {
                                    *failed_tweets.lock().await += 1;
                                    red_ln!("🚨 Failed to delete tweet {}: {}", id, te);
                                }
                            }
                            _ => {
                                *failed_tweets.lock().await += 1;
                                red_ln!("🚨 Failed to delete tweet {}: {}", id, e);
                            }
                        }
                    }
                }
            } else {
                *deleted_tweets.lock().await += 1;
            }
            pb.lock().await.inc(1);
        }
    }))
    .buffer_unordered(opts.max_tasks)
    .collect::<Vec<_>>();
    tasks.await;
    pb.lock().await.finish();
    green_ln!("✅ Done! Deleted {} tweets", deleted_tweets.lock().await);
    if *failed_tweets.lock().await > 0 {
        red_ln!("🚨 {} tweets failed to delete", failed_tweets.lock().await);
    }
}
