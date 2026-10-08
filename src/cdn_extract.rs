use std::collections::HashMap;

#[derive(Default)]
pub enum CdnAvatarExtract<'a> {
    List(&'a Vec<String>),
    Single(&'a String),
    #[default]
    None,
}

pub async fn extract_cdn_avatars<'a, T: 'a, F, M, Fut>(items: &'a Vec<T>, url_extractor: F, avatar_mapper: M, cdn: &mut HashMap<String, String>)
where
    F: Fn(&'a T) -> CdnAvatarExtract<'a>,
    M: Fn(&'a String) -> Fut,
    Fut: Future<Output = Option<(String, String)>>,
{
    for item in items {
        match url_extractor(item) {
            CdnAvatarExtract::List(urls) => {
                for url in urls {
                    if let Some((key, value)) = avatar_mapper(url).await {
                        cdn.insert(key, value);
                    }
                }
            }
            CdnAvatarExtract::Single(url) => {
                if let Some((key, value)) = avatar_mapper(url).await {
                    cdn.insert(key, value);
                }
            }
            CdnAvatarExtract::None => {}
        }
    }
}

pub fn extract_cdn_url(avatar: &String) -> Option<(&str, &str, &str)> {
    if avatar.starts_with(":cdn:") {
        let data = avatar.split(':').collect::<Vec<&str>>();
        if data.len() == 5 {
            let id = data.get(2).unwrap();
            let ext = data.get(3).unwrap();
            let access = data.get(4).unwrap();

            return Some((id, ext, access));
        }
    }
    None
}