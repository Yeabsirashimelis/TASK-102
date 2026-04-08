DELETE FROM notification_templates WHERE code IN ('moderation.approved', 'moderation.rejected', 'comment.reply', 'system.announcement');
DELETE FROM role_permissions WHERE permission_point_id IN (
    SELECT id FROM permission_points WHERE code LIKE 'notification.%'
);
DELETE FROM permission_points WHERE code LIKE 'notification.%';
