DELETE FROM api_capabilities WHERE path_pattern LIKE '/api/v1/orders/*/reversals%'
   OR path_pattern LIKE '/api/v1/orders/*/returns'
   OR path_pattern LIKE '/api/v1/orders/*/exchanges'
   OR path_pattern LIKE '/api/v1/registers/%'
   OR path_pattern LIKE '/api/v1/datasets/*/rollback%';
