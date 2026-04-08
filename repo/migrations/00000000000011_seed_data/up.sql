-- Bootstrap: System Administrator role
INSERT INTO roles (id, name, description, data_scope, scope_value, is_active)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'System Administrator',
    'Full system access, manages roles and permissions',
    'department',
    NULL,
    TRUE
);

-- Store Manager role
INSERT INTO roles (id, name, description, data_scope, scope_value, is_active)
VALUES (
    'a0000000-0000-0000-0000-000000000002',
    'Store Manager',
    'Manages store operations, reconciliation approvals',
    'location',
    NULL,
    TRUE
);

-- Cashier role
INSERT INTO roles (id, name, description, data_scope, scope_value, is_active)
VALUES (
    'a0000000-0000-0000-0000-000000000003',
    'Cashier',
    'POS transactions and register operations',
    'individual',
    NULL,
    TRUE
);

-- Analyst role
INSERT INTO roles (id, name, description, data_scope, scope_value, is_active)
VALUES (
    'a0000000-0000-0000-0000-000000000004',
    'Analyst',
    'Reporting, dataset queries, exports',
    'department',
    NULL,
    TRUE
);

-- Coordinator role
INSERT INTO roles (id, name, description, data_scope, scope_value, is_active)
VALUES (
    'a0000000-0000-0000-0000-000000000005',
    'Content Coordinator',
    'Participant management, notifications, templates',
    'department',
    NULL,
    TRUE
);

-- Core permission points
INSERT INTO permission_points (id, code, description, requires_approval) VALUES
    ('b0000000-0000-0000-0000-000000000001', 'role.list', 'List roles', FALSE),
    ('b0000000-0000-0000-0000-000000000002', 'role.read', 'View role details', FALSE),
    ('b0000000-0000-0000-0000-000000000003', 'role.create', 'Create roles', FALSE),
    ('b0000000-0000-0000-0000-000000000004', 'role.update', 'Update roles', FALSE),
    ('b0000000-0000-0000-0000-000000000005', 'role.delete', 'Delete roles', FALSE),
    ('b0000000-0000-0000-0000-000000000006', 'permission.list', 'List permission points', FALSE),
    ('b0000000-0000-0000-0000-000000000007', 'permission.read', 'View permission details', FALSE),
    ('b0000000-0000-0000-0000-000000000008', 'permission.create', 'Create permission points', FALSE),
    ('b0000000-0000-0000-0000-000000000009', 'permission.update', 'Update permission points', FALSE),
    ('b0000000-0000-0000-0000-000000000010', 'permission.delete', 'Delete permission points', FALSE),
    ('b0000000-0000-0000-0000-000000000011', 'role_permission.bind', 'Bind permission to role', FALSE),
    ('b0000000-0000-0000-0000-000000000012', 'role_permission.unbind', 'Unbind permission from role', FALSE),
    ('b0000000-0000-0000-0000-000000000013', 'user.create', 'Create users', FALSE),
    ('b0000000-0000-0000-0000-000000000014', 'user.list', 'List users', FALSE),
    ('b0000000-0000-0000-0000-000000000015', 'user.read', 'View user details', FALSE),
    ('b0000000-0000-0000-0000-000000000016', 'api_cap.list', 'List API capabilities', FALSE),
    ('b0000000-0000-0000-0000-000000000017', 'api_cap.read', 'View API capability', FALSE),
    ('b0000000-0000-0000-0000-000000000018', 'api_cap.create', 'Create API capability', FALSE),
    ('b0000000-0000-0000-0000-000000000019', 'api_cap.update', 'Update API capability', FALSE),
    ('b0000000-0000-0000-0000-000000000020', 'api_cap.delete', 'Delete API capability', FALSE),
    ('b0000000-0000-0000-0000-000000000021', 'menu_scope.list', 'List menu scopes', FALSE),
    ('b0000000-0000-0000-0000-000000000022', 'menu_scope.read', 'View menu scope', FALSE),
    ('b0000000-0000-0000-0000-000000000023', 'menu_scope.create', 'Create menu scope', FALSE),
    ('b0000000-0000-0000-0000-000000000024', 'menu_scope.update', 'Update menu scope', FALSE),
    ('b0000000-0000-0000-0000-000000000025', 'menu_scope.delete', 'Delete menu scope', FALSE),
    ('b0000000-0000-0000-0000-000000000026', 'delegation.list', 'List delegations', FALSE),
    ('b0000000-0000-0000-0000-000000000027', 'delegation.create', 'Create delegation', FALSE),
    ('b0000000-0000-0000-0000-000000000028', 'delegation.revoke', 'Revoke delegation', FALSE),
    ('b0000000-0000-0000-0000-000000000029', 'approval.list', 'List approval requests', FALSE),
    ('b0000000-0000-0000-0000-000000000030', 'approval.read', 'View approval request', FALSE),
    ('b0000000-0000-0000-0000-000000000031', 'approval.decide', 'Approve or reject requests', FALSE),
    -- Critical actions requiring approval
    ('b0000000-0000-0000-0000-000000000032', 'refund.create', 'Create refund', TRUE),
    ('b0000000-0000-0000-0000-000000000033', 'reversal.create', 'Create reversal', TRUE),
    ('b0000000-0000-0000-0000-000000000034', 'dataset.rollback', 'Rollback dataset', TRUE),
    ('b0000000-0000-0000-0000-000000000035', 'export.bulk', 'Bulk data export', TRUE);

-- Bind all permissions to System Administrator role
INSERT INTO role_permissions (role_id, permission_point_id)
SELECT
    'a0000000-0000-0000-0000-000000000001',
    id
FROM permission_points;

-- Approval policies for critical actions (System Administrator or Store Manager must approve)
INSERT INTO approval_policies (permission_point_id, min_approvers, approver_role_id) VALUES
    ('b0000000-0000-0000-0000-000000000032', 1, 'a0000000-0000-0000-0000-000000000002'),
    ('b0000000-0000-0000-0000-000000000033', 1, 'a0000000-0000-0000-0000-000000000002'),
    ('b0000000-0000-0000-0000-000000000034', 1, 'a0000000-0000-0000-0000-000000000001'),
    ('b0000000-0000-0000-0000-000000000035', 1, 'a0000000-0000-0000-0000-000000000001');
