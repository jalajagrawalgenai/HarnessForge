use forge_cloud::aws::AwsIntegration;
use forge_cloud::azure::AzureIntegration;
use forge_cloud::deploy::{CloudDeploy, CloudProvider};
use forge_cloud::gcp::GcpIntegration;

#[test]
fn test_aws_region() {
    let aws = AwsIntegration::new("us-east-1");
    assert_eq!(aws.region(), "us-east-1");
}

#[test]
fn test_azure_region() {
    let az = AzureIntegration::new("westeurope");
    assert_eq!(az.region(), "westeurope");
}

#[test]
fn test_gcp_region() {
    let gcp = GcpIntegration::new("us-central1");
    assert_eq!(gcp.region(), "us-central1");
}

#[test]
fn test_cloud_deploy() {
    let result = CloudDeploy::deploy(CloudProvider::Aws, "{}");
    assert!(!result.is_empty());
}
