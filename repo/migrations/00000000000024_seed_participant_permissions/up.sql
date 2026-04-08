INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000060', 'participant.create', 'Create participants', FALSE),
    ('b0000000-0000-0000-0000-000000000061', 'participant.read', 'View participants', FALSE),
    ('b0000000-0000-0000-0000-000000000062', 'participant.update', 'Update participants', FALSE),
    ('b0000000-0000-0000-0000-000000000063', 'participant.delete', 'Deactivate participants', FALSE),
    ('b0000000-0000-0000-0000-000000000064', 'participant.search', 'Search and filter participants', FALSE),
    ('b0000000-0000-0000-0000-000000000065', 'participant.bulk', 'Bulk operations on participants', FALSE),
    ('b0000000-0000-0000-0000-000000000066', 'participant.tag', 'Manage participant tags', FALSE),
    ('b0000000-0000-0000-0000-000000000067', 'participant.attach', 'Upload file attachments', FALSE),
    ('b0000000-0000-0000-0000-000000000068', 'team.create', 'Create teams', FALSE),
    ('b0000000-0000-0000-0000-000000000069', 'team.read', 'View teams', FALSE),
    ('b0000000-0000-0000-0000-000000000070', 'team.update', 'Update teams', FALSE),
    ('b0000000-0000-0000-0000-000000000071', 'team.delete', 'Deactivate teams', FALSE),
    ('b0000000-0000-0000-0000-000000000072', 'team.manage_members', 'Add/remove team members', FALSE);

-- Bind to System Administrator
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id
FROM permission_points WHERE code LIKE 'participant.%' OR code LIKE 'team.%'
ON CONFLICT DO NOTHING;

-- Bind to Store Manager
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000002', id
FROM permission_points WHERE code LIKE 'participant.%' OR code LIKE 'team.%'
ON CONFLICT DO NOTHING;

-- Bind to Content Coordinator
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000005', id
FROM permission_points WHERE code LIKE 'participant.%' OR code LIKE 'team.%'
ON CONFLICT DO NOTHING;
