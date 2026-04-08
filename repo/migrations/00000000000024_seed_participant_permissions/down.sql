DELETE FROM role_permissions WHERE permission_point_id IN (
    SELECT id FROM permission_points WHERE code LIKE 'participant.%' OR code LIKE 'team.%'
);
DELETE FROM permission_points WHERE code LIKE 'participant.%' OR code LIKE 'team.%';
