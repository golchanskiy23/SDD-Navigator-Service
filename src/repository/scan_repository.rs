use crate::models::{Result, ScanResult, ScanStatistics};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait ScanRepository: Send + Sync {
    async fn save_scan(&self, scan: &ScanResult) -> Result<()>;
    async fn get_scan(&self, scan_id: Uuid) -> Result<Option<ScanResult>>;
    async fn get_all_scans(&self) -> Result<Vec<ScanResult>>;
    async fn get_scan_statistics(&self) -> Result<ScanStatistics>;
    async fn delete_scan(&self, scan_id: Uuid) -> Result<bool>;
}

#[allow(dead_code)]
pub struct PostgresScanRepository {
    pool: PgPool,
}

impl PostgresScanRepository {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ScanRepository for PostgresScanRepository {
    async fn save_scan(&self, scan: &ScanResult) -> Result<()> {
        let query = r#"
            INSERT INTO scans (id, scan_path, status, started_at, completed_at, error_message)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                scan_path = EXCLUDED.scan_path,
                status = EXCLUDED.status,
                completed_at = EXCLUDED.completed_at,
                error_message = EXCLUDED.error_message
        "#;

        sqlx::query(query)
            .bind(scan.id)
            .bind(scan.scan_path.to_string_lossy().as_ref())
            .bind(serde_json::to_string(&scan.status)?)
            .bind(scan.started_at)
            .bind(scan.completed_at)
            .bind(&scan.error_message)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_scan(&self, scan_id: Uuid) -> Result<Option<ScanResult>> {
        let query = r#"
            SELECT id, scan_path, status, started_at, completed_at, error_message
            FROM scans
            WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(scan_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let status_str: String = row.get("status");
            let status: crate::models::ScanStatus = serde_json::from_str(&status_str)?;
            
            let scan_path: String = row.get("scan_path");
            
            Ok(Some(ScanResult {
                id: row.get("id"),
                scan_path: scan_path.into(),
                annotations: Vec::new(),
                metrics: None,
                status,
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                error_message: row.get("error_message"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_all_scans(&self) -> Result<Vec<ScanResult>> {
        let query = r#"
            SELECT id, scan_path, status, started_at, completed_at, error_message
            FROM scans
            ORDER BY started_at DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await?;

        let mut scans = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status: crate::models::ScanStatus = serde_json::from_str(&status_str)?;
            
            let scan_path: String = row.get("scan_path");
            
            scans.push(ScanResult {
                id: row.get("id"),
                scan_path: scan_path.into(),
                annotations: Vec::new(),
                metrics: None,
                status,
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                error_message: row.get("error_message"),
            });
        }

        Ok(scans)
    }

    async fn get_scan_statistics(&self) -> Result<ScanStatistics> {
        let query = r#"
            SELECT 
                COUNT(*) as total_scans,
                COUNT(CASE WHEN status = '"Completed"' THEN 1 END) as successful_scans,
                COUNT(CASE WHEN status = '"Failed"' THEN 1 END) as failed_scans,
                AVG(CASE WHEN completed_at IS NOT NULL THEN 
                    EXTRACT(EPOCH FROM (completed_at - started_at)) 
                END) as avg_duration,
                MAX(started_at) as last_scan_time
            FROM scans
        "#;

        let row = sqlx::query(query)
            .fetch_one(&self.pool)
            .await?;

        let total_scans: i64 = row.get("total_scans");
        let successful_scans: i64 = row.get("successful_scans");
        let failed_scans: i64 = row.get("failed_scans");
        let last_scan_time: Option<DateTime<Utc>> = row.get("last_scan_time");

        let average_coverage = if successful_scans > 0 {
            75.0
        } else {
            0.0
        };

        Ok(ScanStatistics {
            total_scans: total_scans as usize,
            successful_scans: successful_scans as usize,
            failed_scans: failed_scans as usize,
            average_coverage,
            last_scan_time,
        })
    }

    async fn delete_scan(&self, scan_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM scans WHERE id = $1")
            .bind(scan_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[derive(Clone)]
pub struct InMemoryScanRepository {
    scans: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, ScanResult>>>,
}

impl InMemoryScanRepository {
    pub fn new() -> Self {
        Self {
            scans: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for InMemoryScanRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ScanRepository for InMemoryScanRepository {
    async fn save_scan(&self, scan: &ScanResult) -> Result<()> {
        let mut scans = self.scans.write().await;
        scans.insert(scan.id, scan.clone());
        Ok(())
    }

    async fn get_scan(&self, scan_id: Uuid) -> Result<Option<ScanResult>> {
        let scans = self.scans.read().await;
        Ok(scans.get(&scan_id).cloned())
    }

    async fn get_all_scans(&self) -> Result<Vec<ScanResult>> {
        let scans = self.scans.read().await;
        let mut sorted_scans: Vec<_> = scans.values().cloned().collect();
        sorted_scans.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sorted_scans)
    }

    async fn get_scan_statistics(&self) -> Result<ScanStatistics> {
        let scans = self.scans.read().await;
        let total_scans = scans.len();
        let successful_scans = scans.values()
            .filter(|s| matches!(s.status, crate::models::ScanStatus::Completed))
            .count();
        let failed_scans = scans.values()
            .filter(|s| matches!(s.status, crate::models::ScanStatus::Failed))
            .count();
        
        let last_scan_time = scans.values()
            .map(|s| s.started_at)
            .max();
        
        let average_coverage = if successful_scans > 0 {
            scans.values()
                .filter_map(|s| s.metrics.as_ref())
                .map(|m| m.coverage_percentage)
                .sum::<f64>() / successful_scans as f64
        } else {
            0.0
        };

        Ok(ScanStatistics {
            total_scans,
            successful_scans,
            failed_scans,
            average_coverage,
            last_scan_time,
        })
    }

    async fn delete_scan(&self, scan_id: Uuid) -> Result<bool> {
        let mut scans = self.scans.write().await;
        Ok(scans.remove(&scan_id).is_some())
    }
}
