CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS scans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scan_path TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('InProgress', 'Completed', 'Failed')),
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS annotations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    requirement_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    context TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_annotation UNIQUE (scan_id, requirement_id, file_path, line_number)
);

CREATE TABLE IF NOT EXISTS coverage_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    total_requirements INTEGER NOT NULL DEFAULT 0,
    covered_requirements INTEGER NOT NULL DEFAULT 0,
    coverage_percentage DECIMAL(5,2) NOT NULL DEFAULT 0.00,
    uncovered_requirements JSONB,
    file_coverage JSONB,
    calculated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_coverage UNIQUE (scan_id)
);

CREATE TABLE IF NOT EXISTS coverage_reports (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scan_id UUID NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
    metrics_id UUID NOT NULL REFERENCES coverage_metrics(id) ON DELETE CASCADE,
    recommendations JSONB NOT NULL,
    generated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_report UNIQUE (scan_id)
);

CREATE INDEX IF NOT EXISTS idx_annotations_scan_id ON annotations(scan_id);
CREATE INDEX IF NOT EXISTS idx_annotations_requirement_id ON annotations(requirement_id);
CREATE INDEX IF NOT EXISTS idx_annotations_file_path ON annotations(file_path);
CREATE INDEX IF NOT EXISTS idx_scans_status ON scans(status);
CREATE INDEX IF NOT EXISTS idx_scans_started_at ON scans(started_at);
CREATE INDEX IF NOT EXISTS idx_coverage_metrics_scan_id ON coverage_metrics(scan_id);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_scans_updated_at 
    BEFORE UPDATE ON scans 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE OR REPLACE VIEW scan_statistics AS
SELECT 
    COUNT(*) as total_scans,
    COUNT(CASE WHEN status = 'Completed' THEN 1 END) as successful_scans,
    COUNT(CASE WHEN status = 'Failed' THEN 1 END) as failed_scans,
    COUNT(CASE WHEN status = 'InProgress' THEN 1 END) as in_progress_scans,
    AVG(CASE 
        WHEN status = 'Completed' AND completed_at IS NOT NULL 
        THEN EXTRACT(EPOCH FROM (completed_at - started_at)) 
    END) as avg_duration_seconds,
    MAX(started_at) as last_scan_time
FROM scans;

CREATE OR REPLACE FUNCTION get_coverage_trend(days_back INTEGER DEFAULT 30)
RETURNS TABLE (
    date DATE,
    avg_coverage DECIMAL(5,2),
    total_scans BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        DATE(cm.calculated_at) as date,
        AVG(cm.coverage_percentage) as avg_coverage,
        COUNT(*) as total_scans
    FROM coverage_metrics cm
    JOIN scans s ON cm.scan_id = s.id
    WHERE s.status = 'Completed' 
    AND cm.calculated_at >= NOW() - INTERVAL '1 day' * days_back
    GROUP BY DATE(cm.calculated_at)
    ORDER BY date DESC;
END;
$$ LANGUAGE plpgsql;

DO $$
  DECLARE
      sample_scan_id UUID;
  BEGIN
      IF (SELECT COUNT(*) FROM scans) = 0 THEN
          INSERT INTO scans (id, scan_path, status, started_at, completed_at)
          VALUES (
              uuid_generate_v4(),
              '/sample/project',
              'Completed',
              NOW() - INTERVAL '1 hour',
              NOW() - INTERVAL '55 minutes'
          ) RETURNING id INTO sample_scan_id;
          
          INSERT INTO annotations (scan_id, requirement_id, file_path, line_number, content)
          VALUES 
              (sample_scan_id, 'REQ-001', '/sample/project/src/main.rs', 10, '// @req(REQ-001) Main function'),
              (sample_scan_id, 'REQ-002', '/sample/project/src/utils.rs', 5, '// @req(REQ-002) Utility function'),
              (sample_scan_id, 'REQ-003', '/sample/project/tests/test_main.rs', 15, '// @req(REQ-003) Test case');
              
          INSERT INTO coverage_metrics (
              scan_id, total_requirements, covered_requirements, coverage_percentage,
              uncovered_requirements, file_coverage
          ) VALUES (
              sample_scan_id, 5, 3, 60.00,
              '["REQ-004", "REQ-005"]'::JSONB,
              '{
                  "/sample/project/src/main.rs": {"requirement_count": 1, "requirements": ["REQ-001"]},
                  "/sample/project/src/utils.rs": {"requirement_count": 1, "requirements": ["REQ-002"]},
                  "/sample/project/tests/test_main.rs": {"requirement_count": 1, "requirements": ["REQ-003"]}
              }'::JSONB
          );
      END IF;
  END $$;
