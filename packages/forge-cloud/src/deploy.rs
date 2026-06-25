pub enum CloudProvider { Aws, Azure, Gcp }

pub struct CloudDeploy;

impl CloudDeploy {
    pub fn deploy(_provider: CloudProvider, _config: &str) -> String {
        "deployment initiated".into()
    }
}
