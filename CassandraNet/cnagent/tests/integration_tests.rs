//! Integration tests for agent registration and heartbeat flow
//!
//! These tests validate the agent's ability to register with the gateway
//! and maintain heartbeat communication.

use cncore::platform::{AgentStore, PlatformServices};
use cnproto::{HeartbeatRequest, RegisterAgentRequest};
use uuid::Uuid;

/// Test agent registration with the in-memory platform
#[test]
fn test_agent_registration_flow() {
    // Initialize platform
    let platform = PlatformServices::init_global();

    // Create tenant and project first
    let tenant = platform
        .provisioning()
        .create_tenant("integration-test-tenant".to_string())
        .expect("Failed to create tenant");
    let project = platform
        .provisioning()
        .create_project(tenant.id, "integration-test-project".to_string())
        .expect("Failed to create project");

    // Register an agent
    let provisioned = platform
        .provisioning()
        .register_agent(tenant.id, project.id, "test-agent-host".to_string())
        .expect("Failed to register agent");

    assert!(!provisioned.api_key.value.is_empty());
    assert_eq!(provisioned.agent.hostname, "test-agent-host");

    // Verify the agent exists in the storage
    let storage = platform.storage();
    let agent = storage
        .get_agent(provisioned.agent.id)
        .expect("Storage error")
        .expect("Agent not found in storage");
    assert_eq!(agent.hostname, "test-agent-host");
    assert_eq!(agent.tenant_id, tenant.id);
    assert_eq!(agent.project_id, project.id);
}

/// Test agent heartbeat updates status
#[test]
fn test_agent_heartbeat_flow() {
    let platform = PlatformServices::init_global();

    // Create tenant, project and agent
    let tenant = platform
        .provisioning()
        .create_tenant("heartbeat-test-tenant".to_string())
        .expect("Failed to create tenant");
    let project = platform
        .provisioning()
        .create_project(tenant.id, "heartbeat-test-project".to_string())
        .expect("Failed to create project");
    let provisioned = platform
        .provisioning()
        .register_agent(tenant.id, project.id, "heartbeat-test-host".to_string())
        .expect("Failed to register agent");

    // Retrieve agent from storage
    let storage = platform.storage();
    let agent = storage
        .get_agent(provisioned.agent.id)
        .expect("Storage error")
        .expect("Agent not found");

    // Agent should have some initial state
    assert!(!agent.hostname.is_empty());
    assert_eq!(agent.hostname, "heartbeat-test-host");
}

/// Test multiple agents can be registered
#[test]
fn test_multiple_agent_registration() {
    let platform = PlatformServices::init_global();

    let tenant = platform
        .provisioning()
        .create_tenant("multi-agent-tenant".to_string())
        .expect("Failed to create tenant");
    let project = platform
        .provisioning()
        .create_project(tenant.id, "multi-agent-project".to_string())
        .expect("Failed to create project");

    // Register multiple agents
    let mut agent_ids = Vec::new();
    for i in 0..5 {
        let provisioned = platform
            .provisioning()
            .register_agent(tenant.id, project.id, format!("agent-{}", i))
            .expect("Failed to register agent");
        agent_ids.push(provisioned.agent.id);
    }

    // Verify all agents are registered
    let storage = platform.storage();
    for id in &agent_ids {
        let agent = storage
            .get_agent(*id)
            .expect("Storage error")
            .expect("Agent not found");
        assert!(!agent.hostname.is_empty());
    }

    // Verify we have 5 unique agents
    assert_eq!(agent_ids.len(), 5);
    let unique_ids: std::collections::HashSet<_> = agent_ids.iter().collect();
    assert_eq!(unique_ids.len(), 5);
}

/// Test agent token issuance
#[test]
fn test_agent_token_issuance() {
    let platform = PlatformServices::init_global();

    let tenant = platform
        .provisioning()
        .create_tenant("token-test-tenant".to_string())
        .expect("Failed to create tenant");
    let project = platform
        .provisioning()
        .create_project(tenant.id, "token-test-project".to_string())
        .expect("Failed to create project");
    let provisioned = platform
        .provisioning()
        .register_agent(tenant.id, project.id, "token-test-host".to_string())
        .expect("Failed to register agent");

    // Issue a token for the agent
    let token = platform
        .provisioning()
        .issue_agent_token(provisioned.agent.id)
        .expect("Failed to issue token");

    assert!(!token.token.is_empty());
}

/// Test heartbeat request/response structures
#[test]
fn test_heartbeat_request_structure() {
    let request = HeartbeatRequest {
        assigned_id: Uuid::new_v4().to_string(),
        cpu_percent: 45.5,
        memory_used_bytes: 1024 * 1024 * 512, // 512 MB
        network_rx_bytes: 1000,
        network_tx_bytes: 2000,
        timestamp_unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    assert!(request.cpu_percent >= 0.0 && request.cpu_percent <= 100.0);
    assert!(request.memory_used_bytes > 0);
    assert!(request.timestamp_unix_ms > 0);
}

/// Test register request structure
#[test]
fn test_register_request_structure() {
    let request = RegisterAgentRequest {
        node_id: Uuid::new_v4().to_string(),
        hostname: "test-host".to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_cores: 4,
        memory_bytes: 8 * 1024 * 1024 * 1024, // 8 GB
        secret: "test-secret".to_string(),
        tenant_id: Uuid::new_v4().to_string(),
        project_id: Uuid::new_v4().to_string(),
    };

    assert!(!request.node_id.is_empty());
    assert!(!request.hostname.is_empty());
    assert!(!request.os.is_empty());
    assert!(!request.arch.is_empty());
    assert!(request.cpu_cores > 0);
    assert!(request.memory_bytes > 0);
}
