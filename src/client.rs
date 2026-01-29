use reqwest::Error;
use reqwest::get;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Post {
    pub id: u32,
    pub body: String,
}

pub struct PostClient {
    pub url: String,
}

impl PostClient {
    pub async fn get_posts(&self) -> Result<Vec<Post>, Error> {
        let posts: Vec<Post> = get(&self.url).await?.json().await?;
        Ok(posts)
    }
}
