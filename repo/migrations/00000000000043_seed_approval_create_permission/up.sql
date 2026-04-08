INSERT INTO permission_points (id, code, description, requires_approval)
VALUES ('b0000000-0000-0000-0000-000000000112', 'approval.request.create', 'Create approval requests', FALSE);

-- Bind to System Administrator and Store Manager
INSERT INTO role_permissions (role_id, permission_point_id) VALUES
  ('a0000000-0000-0000-0000-000000000001', 'b0000000-0000-0000-0000-000000000112'),
  ('a0000000-0000-0000-0000-000000000002', 'b0000000-0000-0000-0000-000000000112')
ON CONFLICT DO NOTHING;
