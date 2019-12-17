use serde_json::Error as JSONError;
use reqwest::Error as ReqError;
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::Error as IOError;

use crate::config::StaticConfig;
use crate::post::{Post, PrePosts};

#[inline]
pub fn fetch_new_posts(
    config: &StaticConfig,
) -> Result<Option<Vec<Post>>, FetchPostError> {
    trace!(
        r#"Making request to "{}""#,
        config.endpoints.no_school_posts
    );

    // Get the posts
    let posts: Vec<Post> = reqwest::get(&config.endpoints.no_school_posts)?.json()?;
    trace!("{:#?}", posts);

    // Calculate hash
    let mut hasher = DefaultHasher::new();
    posts.hash(&mut hasher);
    let hash = hasher.finish();

    // Parse previous posts
    let preposts: PrePosts = serde_json::from_reader(File::open(&config.files.previous_posts)?)?;

    if hash != preposts.hash {
        let new_posts = Vec::from(&posts[0..posts.len() - preposts.posts.len()]);

        // Update the file
        fs::write(
            &config.files.previous_posts,
            serde_json::to_vec_pretty(&PrePosts { hash, posts })?,
        )?;
        trace!("Updated previous posts");

        Ok(Some(new_posts))
    } else {
        trace!("No change");

        Ok(None)
    }
}

#[derive(Debug)]
pub enum FetchPostError {
    IO(IOError),
    JSON(JSONError),
    Reqwest(ReqError)
}

impl From<IOError> for FetchPostError {
    fn from(e: IOError) -> Self {
        Self::IO(e)
    }
}

impl From<JSONError> for FetchPostError {
    fn from(e: JSONError) -> Self {
        Self::JSON(e)
    }
}

impl From<ReqError> for FetchPostError {
    fn from(e: ReqError) -> Self {
        Self::Reqwest(e)
    }
}