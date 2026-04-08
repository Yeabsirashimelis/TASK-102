DELETE FROM approval_policies WHERE permission_point_id IN (
    'b0000000-0000-0000-0000-000000000048',
    'b0000000-0000-0000-0000-000000000050',
    'b0000000-0000-0000-0000-000000000052'
);

DELETE FROM role_permissions WHERE permission_point_id IN (
    SELECT id FROM permission_points WHERE code LIKE 'order.%' OR code LIKE 'register.%'
);

DELETE FROM permission_points WHERE code LIKE 'order.%' OR code LIKE 'register.%';
