#[derive(thiserror::Error, Debug)]
pub enum GetIdentitiesError {
    #[error("IsRepoStarred: {}", _0)]
    IsRepoStarred(#[from] UserStaredRepoError),
}

pub async fn get_identities(
    access_token: &str,
    identities: Vec<&crate::Identity>,
) -> Result<bool, GetIdentitiesError> {
    if identities.is_empty() {
        return Ok(false);
    }

    let identities_group_by = crate::utils::group_by(identities, |item| item.key.as_str());
    for (key, values) in identities_group_by {
        match key {
            "github-starred" => {
                let is_user_starred_repos = is_user_starred_any(
                    access_token,
                    values.iter().map(|i| i.value.as_str()).collect(),
                )
                .await?;
                println!("is_user_starred_any_repo: {:?}", is_user_starred_repos);
                return Ok(is_user_starred_repos);
            }
            _ => {}
        }
    }
    Ok(false)
}

// Note: this function receives AbrarNitk/bookrafter or AbrarNitk/auth is starred or not the logged in user

#[derive(thiserror::Error, Debug)]
pub enum UserStaredRepoError {}

pub async fn is_user_starred_any(
    access_token: &str,
    resource_ids: Vec<&str>,
) -> Result<bool, UserStaredRepoError> {
    for resource in resource_ids.into_iter() {
        if is_repo_starred(access_token, resource).await? {
            return Ok(true);
        } else {
            eprintln!("not starred: {}", resource)
        }
    }
    Ok(false)
}

// API Docs: https://docs.github.com/en/rest/activity/starring?apiVersion=2022-11-28#check-if-a-repository-is-starred-by-the-authenticated-user
pub async fn is_repo_starred(
    access_token: &str,
    repo_name: &str,
) -> Result<bool, UserStaredRepoError> {
    let url = format!("https://api.github.com/user/starred/{}", repo_name);

    let response = reqwest::Client::new()
        .get(url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("{}{}", "Bearer ", access_token),
        )
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("bookrafter"),
        )
        .header(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        )
        .send()
        .await;

    // todo: return appropriate error response
    match response {
        Ok(r) => {
            if r.status() == reqwest::StatusCode::NO_CONTENT {
                Ok(true)
            } else {
                eprintln!(
                    "api error repo_name:{},  status {}, body: {:?} ",
                    repo_name,
                    r.status(),
                    r.text().await
                );
                Ok(false)
            }
        }
        Err(e) => {
            eprintln!("api error: {}", e);
            Ok(false)
        }
    }
}

// pub async fn user_starred_repo(access_token: &str) {}

/*
Note: Which API is better to use for the stars check

- https://docs.github.com/en/rest/activity/starring?apiVersion=2022-11-28#list-stargazers
- https://docs.github.com/en/rest/activity/starring?apiVersion=2022-11-28#check-if-a-repository-is-starred-by-the-authenticated-user

API for giving the star
- https://docs.github.com/en/rest/activity/starring?apiVersion=2022-11-28#star-a-repository-for-the-authenticated-user
 */
