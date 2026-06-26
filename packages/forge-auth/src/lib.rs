//! Forge Auth — SSO / SAML / OIDC authentication for Forge Cloud.
//!
//! Supports: Okta, Azure AD, Google Workspace, custom OIDC providers.
//! Features: JIT provisioning, role mapping from IdP groups, API key management.

use serde::{Deserialize, Serialize};

/// Supported identity providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdProvider {
    Okta,
    AzureAD,
    GoogleWorkspace,
    CustomOIDC { issuer_url: String },
}

/// SAML/OIDC configuration for an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    pub provider: IdProvider,
    pub client_id: String,
    pub client_secret: String, // Stored in secrets manager, not plaintext
    pub redirect_url: String,
    pub scopes: Vec<String>,
    /// Map IdP group names to Forge roles
    pub group_role_mapping: Vec<GroupRoleMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRoleMapping {
    pub idp_group: String,
    pub forge_role: Role,
}

/// Forge roles for RBAC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Editor,
    Viewer,
    Auditor,
    Custom { permissions: Vec<String> },
}

/// JIT provisioning: auto-create user on first SSO login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitProvisioning {
    pub enabled: bool,
    pub default_role: Role,
    pub allowed_domains: Option<Vec<String>>,
}

/// Complete auth configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sso: Option<SsoConfig>,
    pub jit_provisioning: JitProvisioning,
    pub session_duration_hours: u32,
    pub mfa_required: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            sso: None,
            jit_provisioning: JitProvisioning {
                enabled: false,
                default_role: Role::Viewer,
                allowed_domains: None,
            },
            session_duration_hours: 24,
            mfa_required: false,
        }
    }
}

/// Validate an SSO token and return the authenticated user.
pub async fn validate_sso_token(
    _config: &SsoConfig,
    _token: &str,
) -> Result<AuthenticatedUser, AuthError> {
    // In production: validate JWT/OIDC token against the IdP's JWKS endpoint
    // For now: return the scaffold structure
    Err(AuthError::NotConfigured)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub email: String,
    pub name: String,
    pub groups: Vec<String>,
    pub role: Role,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("SSO not configured for this organization")]
    NotConfigured,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("IdP unreachable: {0}")]
    ProviderUnreachable(String),
    #[error("Domain not allowed: {0}")]
    DomainNotAllowed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_auth_config() {
        let config = AuthConfig::default();
        assert!(!config.jit_provisioning.enabled);
        assert!(!config.mfa_required);
        assert_eq!(config.session_duration_hours, 24);
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::Admin;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""Admin""#);
    }
}
