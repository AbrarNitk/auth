pub async fn get_identities(token: &str, identities: Vec<&crate::Identity>) -> Result<(), ()> {
    dbg!(token);
    dbg!(&identities);
    if identities.is_empty() {
        return Ok(());
    }
    Ok(())
}
