-- API capabilities for critical mutation endpoints:
-- reversal, register, dataset rollback

INSERT INTO api_capabilities (permission_point_id, http_method, path_pattern, description) VALUES
  -- Reversal endpoints (order.reverse)
  ('b0000000-0000-0000-0000-000000000048', 'POST', '/api/v1/orders/*/reversals', 'Initiate reversal'),
  ('b0000000-0000-0000-0000-000000000048', 'POST', '/api/v1/orders/*/reversals/execute', 'Execute reversal'),
  -- Late reversal (order.reverse_late)
  ('b0000000-0000-0000-0000-000000000052', 'POST', '/api/v1/orders/*/reversals', 'Initiate late reversal'),
  ('b0000000-0000-0000-0000-000000000052', 'POST', '/api/v1/orders/*/reversals/execute', 'Execute late reversal'),
  -- Return/exchange
  ('b0000000-0000-0000-0000-000000000046', 'POST', '/api/v1/orders/*/returns', 'Initiate return'),
  ('b0000000-0000-0000-0000-000000000047', 'POST', '/api/v1/orders/*/exchanges', 'Initiate exchange'),
  -- Register endpoints
  ('b0000000-0000-0000-0000-000000000049', 'POST', '/api/v1/registers/close', 'Close register'),
  ('b0000000-0000-0000-0000-000000000050', 'POST', '/api/v1/registers/closings/*/confirm', 'Confirm variance'),
  ('b0000000-0000-0000-0000-000000000051', 'GET', '/api/v1/registers/closings', 'List closings'),
  ('b0000000-0000-0000-0000-000000000051', 'GET', '/api/v1/registers/closings/*', 'Get closing'),
  -- Dataset rollback
  ('b0000000-0000-0000-0000-000000000034', 'POST', '/api/v1/datasets/*/rollback', 'Request rollback'),
  ('b0000000-0000-0000-0000-000000000034', 'POST', '/api/v1/datasets/*/rollback/execute', 'Execute rollback')
ON CONFLICT (http_method, path_pattern) DO NOTHING;
