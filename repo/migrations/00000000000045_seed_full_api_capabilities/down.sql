DELETE FROM api_capabilities WHERE path_pattern LIKE '/api/v1/participants%'
  OR path_pattern LIKE '/api/v1/teams%'
  OR path_pattern LIKE '/api/v1/datasets%'
  OR path_pattern LIKE '/api/v1/notifications%'
  OR path_pattern LIKE '/api/v1/reports%'
  OR path_pattern LIKE '/api/v1/scheduled-reports%';
