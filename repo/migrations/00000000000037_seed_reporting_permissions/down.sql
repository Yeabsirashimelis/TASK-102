DELETE FROM approval_policies WHERE permission_point_id = 'b0000000-0000-0000-0000-000000000106';
DELETE FROM role_permissions WHERE permission_point_id IN (
    SELECT id FROM permission_points WHERE code LIKE 'report.%'
);
DELETE FROM permission_points WHERE code LIKE 'report.%';
