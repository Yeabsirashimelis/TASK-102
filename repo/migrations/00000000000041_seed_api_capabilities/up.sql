-- Seed baseline api_capabilities for core protected routes.
-- Once capabilities exist for a permission_point, check_permission_for_request
-- will enforce method+path matching (layer 3).

-- Order endpoints
INSERT INTO api_capabilities (permission_point_id, http_method, path_pattern, description) VALUES
  ('b0000000-0000-0000-0000-000000000040', 'POST', '/api/v1/orders', 'Create order'),
  ('b0000000-0000-0000-0000-000000000041', 'GET', '/api/v1/orders', 'List orders'),
  ('b0000000-0000-0000-0000-000000000041', 'GET', '/api/v1/orders/*', 'Get order'),
  ('b0000000-0000-0000-0000-000000000042', 'PUT', '/api/v1/orders/*', 'Update order'),
  ('b0000000-0000-0000-0000-000000000043', 'POST', '/api/v1/orders/*/transition', 'Transition order'),
  ('b0000000-0000-0000-0000-000000000044', 'POST', '/api/v1/orders/*/payments', 'Add payment'),
  ('b0000000-0000-0000-0000-000000000041', 'GET', '/api/v1/orders/*/payments', 'List payments'),
  ('b0000000-0000-0000-0000-000000000045', 'POST', '/api/v1/orders/*/receipts', 'Attach receipt'),

-- Export endpoints
  ('b0000000-0000-0000-0000-000000000105', 'POST', '/api/v1/exports', 'Request export'),
  ('b0000000-0000-0000-0000-000000000105', 'GET', '/api/v1/exports', 'List own exports'),
  ('b0000000-0000-0000-0000-000000000105', 'GET', '/api/v1/exports/*', 'Get export job'),
  ('b0000000-0000-0000-0000-000000000107', 'GET', '/api/v1/exports/*/download', 'Download export'),
  ('b0000000-0000-0000-0000-000000000108', 'GET', '/api/v1/exports/admin', 'Admin list exports'),
  ('b0000000-0000-0000-0000-000000000108', 'POST', '/api/v1/exports/*/complete', 'Complete export'),
  ('b0000000-0000-0000-0000-000000000108', 'POST', '/api/v1/exports/*/fail', 'Fail export'),
  ('b0000000-0000-0000-0000-000000000108', 'PUT', '/api/v1/exports/*/progress', 'Update progress'),

-- Approval endpoints
  ('b0000000-0000-0000-0000-000000000029', 'GET', '/api/v1/approvals', 'List approvals'),
  ('b0000000-0000-0000-0000-000000000030', 'GET', '/api/v1/approvals/*', 'Get approval'),
  ('b0000000-0000-0000-0000-000000000031', 'POST', '/api/v1/approvals/*/approve', 'Approve request'),
  ('b0000000-0000-0000-0000-000000000031', 'POST', '/api/v1/approvals/*/reject', 'Reject request');
