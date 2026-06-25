pub struct GcpIntegration { region: String }

impl GcpIntegration {
    pub fn new(region: &str) -> Self { Self { region: region.into() } }
    pub fn region(&self) -> &str { &self.region }
}
