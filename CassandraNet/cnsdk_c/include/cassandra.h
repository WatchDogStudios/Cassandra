#ifndef CASSANDRA_NET_SDK_H
#define CASSANDRA_NET_SDK_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>


// Common return codes
#define CASS_OK 0
#define CASS_ERR_INVALID -1
#define CASS_ERR_UNAUTHORIZED -2
#define CASS_ERR_INTERNAL -3

// Opaque session handle
typedef struct cass_session {
  void *_private;
} cass_session;

// UUID value (little endian bytes)
typedef struct cass_uuid {
  unsigned char bytes[16];
} cass_uuid;

// Configuration passed to cass_init
typedef struct cass_config {
  const char *api_key;
  const char *gateway_url; // http(s) URL of gateway
} cass_config;

// Initialize SDK; returns 0 on success
int cass_init(const cass_config *cfg);
// Authenticate using API key (placeholder)
int cass_authenticate(const char *api_key);
// Send a custom metric
int cass_send_metric(const char *name, double value);
// Obtain a server session (placeholder)
int cass_get_server_session(cass_session *out_session);
// Shutdown / cleanup
void cass_shutdown();

// Free strings allocated by the SDK
void cass_string_free(char *value);

// Provisioning helpers (in-memory defaults)
int cass_create_tenant(const char *name, cass_uuid *out_id);
int cass_create_project(const cass_uuid *tenant_id, const char *name,
                        cass_uuid *out_id);
int cass_register_agent(const cass_uuid *tenant_id, const cass_uuid *project_id,
                        const char *hostname, cass_uuid *out_agent_id,
                        char **out_api_key);
int cass_issue_agent_token(const cass_uuid *agent_id, char **out_token);

// Orchestration helpers
int cass_schedule_task(const cass_uuid *tenant_id, const char *kind,
                       const char *payload_json, cass_uuid *out_task_id);

#ifdef __cplusplus
}
#endif

#endif // CASSANDRA_NET_SDK_H
