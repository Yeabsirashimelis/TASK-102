INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000110', 'audit.read', 'Query audit trail', FALSE),
    ('b0000000-0000-0000-0000-000000000111', 'system.health', 'View health and metrics', FALSE);

-- System Administrator gets all
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id
FROM permission_points WHERE code IN ('audit.read', 'system.health')
ON CONFLICT DO NOTHING;

-- Store Manager gets health
INSERT INTO role_permissions (role_id, permission_point_id)
VALUES ('a0000000-0000-0000-0000-000000000002', 'b0000000-0000-0000-0000-000000000111')
ON CONFLICT DO NOTHING;
