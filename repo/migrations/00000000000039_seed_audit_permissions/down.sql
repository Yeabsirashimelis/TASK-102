DELETE FROM role_permissions WHERE permission_point_id IN (
    'b0000000-0000-0000-0000-000000000110',
    'b0000000-0000-0000-0000-000000000111'
);
DELETE FROM permission_points WHERE code IN ('audit.read', 'system.health');
