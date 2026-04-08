INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000090', 'notification.template.create', 'Create notification templates', FALSE),
    ('b0000000-0000-0000-0000-000000000091', 'notification.template.read', 'View notification templates', FALSE),
    ('b0000000-0000-0000-0000-000000000092', 'notification.template.update', 'Update notification templates', FALSE),
    ('b0000000-0000-0000-0000-000000000093', 'notification.template.delete', 'Delete notification templates', FALSE),
    ('b0000000-0000-0000-0000-000000000094', 'notification.send', 'Send notifications', FALSE),
    ('b0000000-0000-0000-0000-000000000095', 'notification.broadcast', 'Send system announcements to all users', FALSE),
    ('b0000000-0000-0000-0000-000000000096', 'notification.read', 'View own notifications', FALSE),
    ('b0000000-0000-0000-0000-000000000097', 'notification.admin', 'View all notifications and delivery logs', FALSE),
    ('b0000000-0000-0000-0000-000000000098', 'notification.retry', 'Retry failed notification deliveries', FALSE);

-- System Administrator gets all
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id
FROM permission_points WHERE code LIKE 'notification.%'
ON CONFLICT DO NOTHING;

-- Store Manager gets send, broadcast, read, admin
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000002', id
FROM permission_points WHERE code IN (
    'notification.send', 'notification.broadcast',
    'notification.read', 'notification.admin',
    'notification.template.read', 'notification.retry'
)
ON CONFLICT DO NOTHING;

-- Content Coordinator gets template management + send + read
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000005', id
FROM permission_points WHERE code IN (
    'notification.template.create', 'notification.template.read',
    'notification.template.update', 'notification.send',
    'notification.read'
)
ON CONFLICT DO NOTHING;

-- All other roles get read (own notifications)
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT r.id, 'b0000000-0000-0000-0000-000000000096'
FROM roles r
WHERE r.id NOT IN (
    'a0000000-0000-0000-0000-000000000001',
    'a0000000-0000-0000-0000-000000000002',
    'a0000000-0000-0000-0000-000000000005'
)
ON CONFLICT DO NOTHING;
