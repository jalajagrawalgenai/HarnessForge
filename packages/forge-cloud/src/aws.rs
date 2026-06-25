pub struct AwsIntegration {
    region: String,
}

impl AwsIntegration {
    pub fn new(region: &str) -> Self {
        Self {
            region: region.into(),
        }
    }
    pub fn region(&self) -> &str {
        &self.region
    }
}
