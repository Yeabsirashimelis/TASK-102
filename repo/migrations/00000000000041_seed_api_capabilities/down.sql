DELETE FROM api_capabilities WHERE path_pattern LIKE '/api/v1/orders%'
   OR path_pattern LIKE '/api/v1/exports%'
   OR path_pattern LIKE '/api/v1/approvals%';
