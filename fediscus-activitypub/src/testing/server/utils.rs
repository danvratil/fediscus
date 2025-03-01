use rand::{distr::Alphanumeric, prelude::*};
use url::{ParseError, Url};

/// Just generate random url as object id. In a real project, you probably want to use
/// an url which contains the database id for easy retrieval (or store the random id in db).
pub fn generate_object_id(domain: &str) -> Result<Url, ParseError> {
    let id: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    Url::parse(&format!("http://{}/objects/{}", domain, id))
}
