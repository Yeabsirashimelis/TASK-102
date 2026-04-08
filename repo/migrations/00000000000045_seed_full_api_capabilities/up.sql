-- Comprehensive api_capabilities seed for all protected endpoints.
-- Ensures layer-3 RBAC enforcement for every domain.

-- Participant endpoints
INSERT INTO api_capabilities (permission_point_id, http_method, path_pattern, description) VALUES
  ('b0000000-0000-0000-0000-000000000060', 'POST', '/api/v1/participants', 'Create participant'),
  ('b0000000-0000-0000-0000-000000000061', 'GET', '/api/v1/participants', 'List participants'),
  ('b0000000-0000-0000-0000-000000000061', 'GET', '/api/v1/participants/*', 'Get participant'),
  ('b0000000-0000-0000-0000-000000000062', 'PUT', '/api/v1/participants/*', 'Update participant'),
  ('b0000000-0000-0000-0000-000000000063', 'DELETE', '/api/v1/participants/*', 'Deactivate participant'),
  ('b0000000-0000-0000-0000-000000000065', 'POST', '/api/v1/participants/bulk/tag', 'Bulk tag'),
  ('b0000000-0000-0000-0000-000000000065', 'POST', '/api/v1/participants/bulk/deactivate', 'Bulk deactivate'),
  ('b0000000-0000-0000-0000-000000000066', 'GET', '/api/v1/participants/*/tags', 'Get tags'),
  ('b0000000-0000-0000-0000-000000000066', 'PUT', '/api/v1/participants/*/tags', 'Set tags'),
  ('b0000000-0000-0000-0000-000000000067', 'POST', '/api/v1/participants/*/attachments', 'Upload attachment'),
  ('b0000000-0000-0000-0000-000000000061', 'GET', '/api/v1/participants/*/attachments', 'List attachments'),
  ('b0000000-0000-0000-0000-000000000061', 'GET', '/api/v1/participants/*/attachments/*', 'Download attachment'),
  ('b0000000-0000-0000-0000-000000000067', 'DELETE', '/api/v1/participants/*/attachments/*', 'Delete attachment'),

-- Team endpoints
  ('b0000000-0000-0000-0000-000000000068', 'POST', '/api/v1/teams', 'Create team'),
  ('b0000000-0000-0000-0000-000000000069', 'GET', '/api/v1/teams', 'List teams'),
  ('b0000000-0000-0000-0000-000000000069', 'GET', '/api/v1/teams/*', 'Get team'),
  ('b0000000-0000-0000-0000-000000000070', 'PUT', '/api/v1/teams/*', 'Update team'),
  ('b0000000-0000-0000-0000-000000000071', 'DELETE', '/api/v1/teams/*', 'Deactivate team'),
  ('b0000000-0000-0000-0000-000000000072', 'POST', '/api/v1/teams/*/members', 'Add member'),
  ('b0000000-0000-0000-0000-000000000072', 'DELETE', '/api/v1/teams/*/members/*', 'Remove member'),
  ('b0000000-0000-0000-0000-000000000069', 'GET', '/api/v1/teams/*/members', 'List members'),

-- Dataset endpoints
  ('b0000000-0000-0000-0000-000000000080', 'POST', '/api/v1/datasets', 'Create dataset'),
  ('b0000000-0000-0000-0000-000000000081', 'GET', '/api/v1/datasets', 'List datasets'),
  ('b0000000-0000-0000-0000-000000000081', 'GET', '/api/v1/datasets/*', 'Get dataset'),
  ('b0000000-0000-0000-0000-000000000082', 'PUT', '/api/v1/datasets/*', 'Update dataset'),
  ('b0000000-0000-0000-0000-000000000083', 'DELETE', '/api/v1/datasets/*', 'Deactivate dataset'),
  ('b0000000-0000-0000-0000-000000000084', 'POST', '/api/v1/datasets/*/versions', 'Create version'),
  ('b0000000-0000-0000-0000-000000000085', 'GET', '/api/v1/datasets/*/versions', 'List versions'),
  ('b0000000-0000-0000-0000-000000000085', 'GET', '/api/v1/datasets/*/versions/*', 'Get version'),
  ('b0000000-0000-0000-0000-000000000085', 'GET', '/api/v1/datasets/*/versions/*/lineage', 'Get lineage'),
  ('b0000000-0000-0000-0000-000000000085', 'GET', '/api/v1/datasets/*/versions/*/fields', 'List fields'),
  ('b0000000-0000-0000-0000-000000000087', 'POST', '/api/v1/datasets/*/versions/*/fields', 'Add field'),
  ('b0000000-0000-0000-0000-000000000087', 'PUT', '/api/v1/datasets/*/versions/*/fields/*', 'Update field'),
  ('b0000000-0000-0000-0000-000000000087', 'DELETE', '/api/v1/datasets/*/versions/*/fields/*', 'Delete field'),

-- Notification endpoints
  ('b0000000-0000-0000-0000-000000000094', 'POST', '/api/v1/notifications/send', 'Send templated'),
  ('b0000000-0000-0000-0000-000000000094', 'POST', '/api/v1/notifications/send-direct', 'Send direct'),
  ('b0000000-0000-0000-0000-000000000095', 'POST', '/api/v1/notifications/broadcast', 'Broadcast'),
  ('b0000000-0000-0000-0000-000000000096', 'GET', '/api/v1/notifications/inbox', 'Inbox'),
  ('b0000000-0000-0000-0000-000000000096', 'GET', '/api/v1/notifications/inbox/*', 'Get notification'),
  ('b0000000-0000-0000-0000-000000000096', 'GET', '/api/v1/notifications/inbox/unread-count', 'Unread count'),
  ('b0000000-0000-0000-0000-000000000096', 'POST', '/api/v1/notifications/inbox/read-all', 'Read all'),
  ('b0000000-0000-0000-0000-000000000096', 'POST', '/api/v1/notifications/inbox/*/read', 'Mark read'),
  ('b0000000-0000-0000-0000-000000000097', 'GET', '/api/v1/notifications/admin', 'Admin list'),
  ('b0000000-0000-0000-0000-000000000097', 'GET', '/api/v1/notifications/admin/*/delivery-logs', 'Delivery logs'),
  ('b0000000-0000-0000-0000-000000000098', 'POST', '/api/v1/notifications/admin/*/retry', 'Retry delivery'),

-- Report endpoints
  ('b0000000-0000-0000-0000-000000000100', 'POST', '/api/v1/reports', 'Create report'),
  ('b0000000-0000-0000-0000-000000000101', 'GET', '/api/v1/reports', 'List reports'),
  ('b0000000-0000-0000-0000-000000000101', 'GET', '/api/v1/reports/*', 'Get report'),
  ('b0000000-0000-0000-0000-000000000101', 'GET', '/api/v1/reports/kpi-types', 'KPI types'),
  ('b0000000-0000-0000-0000-000000000101', 'POST', '/api/v1/reports/*/run', 'Run report'),
  ('b0000000-0000-0000-0000-000000000102', 'PUT', '/api/v1/reports/*', 'Update report'),
  ('b0000000-0000-0000-0000-000000000103', 'DELETE', '/api/v1/reports/*', 'Delete report'),
  ('b0000000-0000-0000-0000-000000000104', 'POST', '/api/v1/scheduled-reports', 'Create schedule'),
  ('b0000000-0000-0000-0000-000000000104', 'GET', '/api/v1/scheduled-reports', 'List schedules'),
  ('b0000000-0000-0000-0000-000000000104', 'GET', '/api/v1/scheduled-reports/*', 'Get schedule'),
  ('b0000000-0000-0000-0000-000000000104', 'PUT', '/api/v1/scheduled-reports/*', 'Update schedule'),
  ('b0000000-0000-0000-0000-000000000104', 'DELETE', '/api/v1/scheduled-reports/*', 'Delete schedule'),

-- Approval create
  ('b0000000-0000-0000-0000-000000000112', 'POST', '/api/v1/approvals', 'Create approval request')

ON CONFLICT (http_method, path_pattern) DO NOTHING;
