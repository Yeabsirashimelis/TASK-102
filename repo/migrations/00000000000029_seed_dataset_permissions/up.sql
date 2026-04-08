-- dataset.rollback already seeded in migration 11 (id = b0000000-0000-0000-0000-000000000034)
INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000080', 'dataset.create', 'Create datasets', FALSE),
    ('b0000000-0000-0000-0000-000000000081', 'dataset.read', 'View datasets and versions', FALSE),
    ('b0000000-0000-0000-0000-000000000082', 'dataset.update', 'Update dataset metadata', FALSE),
    ('b0000000-0000-0000-0000-000000000083', 'dataset.delete', 'Deactivate datasets', FALSE),
    ('b0000000-0000-0000-0000-000000000084', 'dataset.version.create', 'Create new dataset version', FALSE),
    ('b0000000-0000-0000-0000-000000000085', 'dataset.version.read', 'View version details and lineage', FALSE),
    ('b0000000-0000-0000-0000-000000000087', 'dataset.field_dict.manage', 'Manage field dictionary entries', FALSE);

-- Bind to System Administrator
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id
FROM permission_points WHERE code LIKE 'dataset.%'
ON CONFLICT DO NOTHING;

-- Bind to Analyst role
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000004', id
FROM permission_points WHERE code LIKE 'dataset.%'
ON CONFLICT DO NOTHING;

-- Bind to Store Manager (read + rollback)
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000002', id
FROM permission_points WHERE code IN (
    'dataset.read', 'dataset.version.read', 'dataset.rollback'
)
ON CONFLICT DO NOTHING;
