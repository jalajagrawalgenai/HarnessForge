pub struct AzureIntegration { region: String }

impl AzureIntegration {
    pub fn new(region: &str) -> Self { Self { region: region.into() } }
    pub fn region(&self) -> &str { &self.region }
}
