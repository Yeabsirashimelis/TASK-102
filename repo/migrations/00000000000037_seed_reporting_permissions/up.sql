INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000100', 'report.create', 'Create report definitions', FALSE),
    ('b0000000-0000-0000-0000-000000000101', 'report.read', 'View reports and run queries', FALSE),
    ('b0000000-0000-0000-0000-000000000102', 'report.update', 'Update report definitions', FALSE),
    ('b0000000-0000-0000-0000-000000000103', 'report.delete', 'Delete report definitions', FALSE),
    ('b0000000-0000-0000-0000-000000000104', 'report.schedule', 'Manage scheduled reports', FALSE),
    ('b0000000-0000-0000-0000-000000000105', 'report.export', 'Request async exports', FALSE),
    ('b0000000-0000-0000-0000-000000000106', 'report.export.bulk', 'Bulk export (>250k rows)', TRUE),
    ('b0000000-0000-0000-0000-000000000107', 'report.export.download', 'Download completed exports', FALSE),
    ('b0000000-0000-0000-0000-000000000108', 'report.export.admin', 'View and manage all export jobs', FALSE);

-- System Administrator gets all
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id
FROM permission_points WHERE code LIKE 'report.%'
ON CONFLICT DO NOTHING;

-- Analyst gets all report permissions
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000004', id
FROM permission_points WHERE code LIKE 'report.%'
ON CONFLICT DO NOTHING;

-- Store Manager gets read + export + download
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000002', id
FROM permission_points WHERE code IN (
    'report.read', 'report.export', 'report.export.download'
)
ON CONFLICT DO NOTHING;

-- Approval policy: bulk export requires System Administrator approval
INSERT INTO approval_policies (permission_point_id, min_approvers, approver_role_id)
VALUES ('b0000000-0000-0000-0000-000000000106', 1, 'a0000000-0000-0000-0000-000000000001');
