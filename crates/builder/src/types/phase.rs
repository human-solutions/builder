use std::str::FromStr;

pub enum Phase {
    PreBuild,
    PostBuild,
}

impl FromStr for Phase {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prebuild" => Ok(Phase::PreBuild),
            "postbuild" => Ok(Phase::PostBuild),
            _ => anyhow::bail!("Invalid phase: {}", s),
        }
    }
}

impl Phase {
    pub fn is_postbuild(&self) -> bool {
        matches!(self, Self::PostBuild)
    }

    pub fn is_pre_build(&self) -> bool {
        matches!(self, Self::PreBuild)
    }
}
