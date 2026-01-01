pub async fn application_ctx(settings: &crate::settings::Settings) -> anyhow::Result<base::Ctx> {
    let profile = settings.service.environment.to_owned();

    let redis_pool = base::pool::redis::create(&settings.redis).await?;
    let postgres_pool = db::pg::pool_with_url(&settings.postgres.url).await?;

    let mut ctx_builder = base::Ctx::builder();
    ctx_builder
        .with_profile(profile)
        .with_postgre(postgres_pool)
        .with_redis(redis_pool)
        // .with_openfga(OpenFGAConfig::from_settings(&settings.openfga))
        .with_dex_settings(settings.dex.clone());

    ctx_builder.build()
}
