-- POS Permission Points
INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000040', 'order.create', 'Create POS orders', FALSE),
    ('b0000000-0000-0000-0000-000000000041', 'order.read', 'View POS orders', FALSE),
    ('b0000000-0000-0000-0000-000000000042', 'order.update', 'Update POS orders', FALSE),
    ('b0000000-0000-0000-0000-000000000043', 'order.transition', 'Transition order state', FALSE),
    ('b0000000-0000-0000-0000-000000000044', 'order.add_payment', 'Record payment on order', FALSE),
    ('b0000000-0000-0000-0000-000000000045', 'order.attach_receipt', 'Attach receipt to order', FALSE),
    ('b0000000-0000-0000-0000-000000000046', 'order.return', 'Initiate return', FALSE),
    ('b0000000-0000-0000-0000-000000000047', 'order.exchange', 'Process exchange', FALSE),
    ('b0000000-0000-0000-0000-000000000048', 'order.reverse', 'Reverse a transaction', TRUE),
    ('b0000000-0000-0000-0000-000000000049', 'register.close', 'Close register for end-of-day', FALSE),
    ('b0000000-0000-0000-0000-000000000050', 'register.confirm_variance', 'Manager confirm variance', TRUE),
    ('b0000000-0000-0000-0000-000000000051', 'register.read', 'View register closings', FALSE),
    ('b0000000-0000-0000-0000-000000000052', 'order.reverse_late', 'Reverse transaction after 24h', TRUE);

-- Bind POS permissions to System Administrator
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id
FROM permission_points WHERE code LIKE 'order.%' OR code LIKE 'register.%'
ON CONFLICT DO NOTHING;

-- Bind cashier-level permissions to Cashier role
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000003', id
FROM permission_points WHERE code IN (
    'order.create', 'order.read', 'order.update', 'order.transition',
    'order.add_payment', 'order.attach_receipt', 'order.return',
    'register.close', 'register.read'
)
ON CONFLICT DO NOTHING;

-- Bind all POS permissions to Store Manager
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT 'a0000000-0000-0000-0000-000000000002', id
FROM permission_points WHERE code LIKE 'order.%' OR code LIKE 'register.%'
ON CONFLICT DO NOTHING;

-- Approval policies: Store Manager and System Administrator can approve
INSERT INTO approval_policies (permission_point_id, min_approvers, approver_role_id) VALUES
    ('b0000000-0000-0000-0000-000000000048', 1, 'a0000000-0000-0000-0000-000000000002'),
    ('b0000000-0000-0000-0000-000000000050', 1, 'a0000000-0000-0000-0000-000000000002'),
    ('b0000000-0000-0000-0000-000000000052', 1, 'a0000000-0000-0000-0000-000000000002'),
    ('b0000000-0000-0000-0000-000000000048', 1, 'a0000000-0000-0000-0000-000000000001'),
    ('b0000000-0000-0000-0000-000000000050', 1, 'a0000000-0000-0000-0000-000000000001'),
    ('b0000000-0000-0000-0000-000000000052', 1, 'a0000000-0000-0000-0000-000000000001');
