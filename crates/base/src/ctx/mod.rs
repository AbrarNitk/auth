pub mod settings;

#[derive(Clone)]
pub struct Ctx {
    pub env: String,
    pub pg_pool: sqlx::PgPool,
    pub redis_pool: base::RedisPool,
    // pub openfga: OpenFGAConfig,
    pub dex: settings::dex::DexSettings,
}

impl Ctx {
    pub fn builder() -> CtxBuilder {
        CtxBuilder::default()
    }
}

#[derive(Default)]
pub struct CtxBuilder {
    pub profile: Option<String>,
    pub pg_pool: Option<sqlx::PgPool>,
    pub redis_pool: Option<base::RedisPool>,
    // pub openfga: Option<OpenFGAConfig>,
    pub dex: Option<settings::dex::DexSettings>,
}

impl CtxBuilder {
    pub fn with_profile(&mut self, profile: String) -> &mut Self {
        self.profile = Some(profile);
        self
    }

    pub fn with_postgre(&mut self, pool: sqlx::PgPool) -> &mut Self {
        self.pg_pool = Some(pool);
        self
    }

    pub fn with_redis(&mut self, pool: base::RedisPool) -> &mut Self {
        self.redis_pool = Some(pool);
        self
    }

    // pub fn with_openfga(&mut self, openfga: OpenFGAConfig) -> &mut Self {
    //     self.openfga = Some(openfga);
    //     self
    // }

    pub fn with_dex_settings(&mut self, dex: settings::dex::DexSettings) -> &mut Self {
        self.dex = Some(dex);
        self
    }

    pub fn build(self) -> anyhow::Result<Ctx> {
        Ok(Ctx {
            env: self.profile.ok_or_else(|| {
                anyhow::anyhow!("`profile` expected to be set in application context")
            })?,
            pg_pool: self.pg_pool.ok_or_else(|| {
                anyhow::anyhow!("`pg_pool` expected to be set in application context")
            })?,
            redis_pool: self.redis_pool.ok_or_else(|| {
                anyhow::anyhow!("`redis_pool` expected to be set in application context")
            })?,
            // openfga: self.openfga.ok_or_else(|| {
            //     anyhow::anyhow!("`openfga` expected to be set in application context")
            // })?,
            dex: self.dex.ok_or_else(|| {
                anyhow::anyhow!("`dex` expected to be set in application context")
            })?,
        })
    }
}
