//! Runs against a real PostgreSQL when `CASS_TEST_DATABASE_URL` points at an
//! empty database (CI provides one); otherwise each test is a no-op.
#![cfg(feature = "db")]

use cncore::platform::{
    AgentJob, BuildRecord, JobOutcome, JobQuery, JobStatus, PlatformError, PortalStore,
    PostgresPortalStore, ScriptRecord,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> Option<PgPool> {
    let url = std::env::var("CASS_TEST_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to test database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("the full migration chain applies to a fresh database");
    Some(pool)
}

#[tokio::test]
async fn migrations_apply_cleanly_and_are_idempotent() {
    let Some(pool) = pool().await else { return };
    // Second run must be a no-op (checksums unchanged, nothing re-applied).
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // 0007 extended the users table created by 0001 and seeded the admin.
    let (email, scopes, active): (String, Vec<String>, bool) = sqlx::query_as(
        "SELECT email, scopes, is_active FROM users
         WHERE id = '00000000-0000-0000-0000-000000000001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(email, "admin@localhost");
    assert!(scopes.contains(&"admin:write".to_string()));
    assert!(active);

    // 0014 added the foreign keys 0006 could not declare.
    let fk_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.table_constraints
         WHERE constraint_type = 'FOREIGN KEY'
           AND table_name IN ('orchestration_assignments', 'ugc_moderation_content', 'messaging_messages')
           AND constraint_name LIKE 'fk_%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(fk_count, 6);

    // Audit rows accept the casts the gateway's Postgres emitter uses.
    sqlx::query(
        "INSERT INTO audit_logs (event_kind, level, actor_id, actor_type, ip_address, payload)
         VALUES ($1, $2, $3::uuid, $4, $5::inet, $6)",
    )
    .bind("test.event")
    .bind("info")
    .bind(Some(Uuid::new_v4().to_string()))
    .bind(Some("user"))
    .bind(None::<String>)
    .bind(json!({"ok": true}))
    .execute(&pool)
    .await
    .expect("audit insert with actor");
}

#[tokio::test]
async fn portal_store_round_trips() {
    let Some(pool) = pool().await else { return };
    let store = PostgresPortalStore::new(pool);
    let now = chrono::Utc::now();

    let build = BuildRecord {
        id: Uuid::new_v4(),
        name: "server".into(),
        version: "1.0.0".into(),
        platform: "linux".into(),
        architecture: "x64".into(),
        filename: "server.tar.gz".into(),
        size_bytes: 42,
        checksum_sha256: "ab".repeat(32),
        uploaded_by: Some("tester".into()),
        metadata: [("channel".to_string(), "beta".to_string())].into(),
        created_at: now,
    };
    store.insert_build(build.clone()).await.unwrap();
    let fetched = store.get_build(build.id).await.unwrap().unwrap();
    assert_eq!(fetched.metadata, build.metadata);
    assert_eq!(fetched.size_bytes, 42);
    assert!(store
        .list_builds()
        .await
        .unwrap()
        .iter()
        .any(|b| b.id == build.id));

    let mut script = ScriptRecord {
        id: Uuid::new_v4(),
        name: "uptime".into(),
        language: "bash".into(),
        content: "uptime".into(),
        description: None,
        tags: vec!["ops".into()],
        version: 1,
        created_by: None,
        created_at: now,
        updated_at: now,
    };
    store.save_script(script.clone()).await.unwrap();
    script.version = 2;
    script.content = "uptime -p".into();
    store.save_script(script.clone()).await.unwrap();
    let saved = store.get_script(script.id).await.unwrap().unwrap();
    assert_eq!((saved.version, saved.content.as_str()), (2, "uptime -p"));
    assert_eq!(saved.tags, vec!["ops".to_string()]);

    let agent = format!("agent-{}", Uuid::new_v4());
    let mut older = AgentJob::new(&agent, "exec", json!({"command": "true"}));
    older.created_at = now - chrono::Duration::seconds(10);
    let mut scripted = AgentJob::new(&agent, "script", json!({}));
    scripted.script_id = Some(script.id);
    store
        .insert_jobs(vec![scripted.clone(), older.clone()])
        .await
        .unwrap();

    let claimed = store.claim_jobs(&agent, 10).await.unwrap();
    let ids: Vec<_> = claimed.iter().map(|j| j.id).collect();
    assert_eq!(ids, vec![older.id, scripted.id], "oldest first");
    assert!(claimed.iter().all(|j| j.status == JobStatus::Dispatched));
    assert!(store.claim_jobs(&agent, 10).await.unwrap().is_empty());

    let stolen = store
        .complete_job(older.id, "someone-else", JobOutcome::default())
        .await;
    assert!(matches!(stolen, Err(PlatformError::NotFound(_))));
    let done = store
        .complete_job(
            older.id,
            &agent,
            JobOutcome {
                success: false,
                exit_code: Some(2),
                stderr: "boom".into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(done.status, JobStatus::Failed);
    assert_eq!(done.exit_code, Some(2));

    let by_script = store
        .list_jobs(JobQuery {
            script_id: Some(script.id),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(by_script.len(), 1);
    let limited = store
        .list_jobs(JobQuery {
            agent_id: Some(agent.clone()),
            limit: Some(1),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(limited.len(), 1);

    // Deleting a script detaches its runs instead of deleting them.
    assert!(store.delete_script(script.id).await.unwrap());
    let detached = store
        .list_jobs(JobQuery {
            agent_id: Some(agent.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(detached.iter().all(|j| j.script_id.is_none()));
    assert_eq!(detached.len(), 2);

    store
        .put_agent_config(&agent, json!({"heartbeat_interval_seconds": 15}))
        .await
        .unwrap();
    store
        .put_agent_config(&agent, json!({"heartbeat_interval_seconds": 20}))
        .await
        .unwrap();
    assert_eq!(
        store.get_agent_config(&agent).await.unwrap(),
        Some(json!({"heartbeat_interval_seconds": 20}))
    );
    store
        .put_settings(json!({"default_heartbeat_interval": 10}))
        .await
        .unwrap();
    assert_eq!(
        store.get_settings().await.unwrap(),
        Some(json!({"default_heartbeat_interval": 10}))
    );

    assert!(store.delete_build(build.id).await.unwrap());
    assert!(!store.delete_build(build.id).await.unwrap());
}
