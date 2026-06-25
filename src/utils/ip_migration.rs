/// 启动时迁移 IP 设置数据格式：旧格式 string[] → 新格式 [{ip,remark}][]
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, Set, IntoActiveModel};

pub async fn migrate_ip_settings_format(db: &DatabaseConnection) {
    use crate::models::entity::settings;

    // 只需迁移 ip_whitelist 和 ip_blacklist
    for key in &["ip_whitelist", "ip_blacklist"] {
        let rows = settings::Entity::find()
            .filter(settings::Column::Key.eq(*key))
            .all(db)
            .await
            .unwrap_or_default();

        for row in rows {
            let value = &row.value;
            // 跳过空值
            if value.is_empty() {
                continue;
            }
            // 尝试解析旧格式 ["ip1", "ip2"]
            if let Ok(ips) = serde_json::from_str::<Vec<String>>(value) {
                if ips.is_empty() { continue; }
                // 检查是否已经是新格式（第一个元素是对象而非字符串）
                if serde_json::from_str::<Vec<serde_json::Value>>(value)
                    .map(|arr| arr.first().map(|v| v.is_object()).unwrap_or(false))
                    .unwrap_or(false)
                {
                    continue; // 已是新格式，跳过
                }

                // 转换为新格式
                use crate::utils::ip_utils;
                let new_list: Vec<serde_json::Value> = ips.into_iter()
                    .filter(|ip| ip_utils::is_valid_ip(ip))
                    .map(|ip| serde_json::json!({ "ip": ip, "remark": "" }))
                    .collect();

                if let Ok(new_json) = serde_json::to_string(&new_list) {
                    let mut active = row.into_active_model();
                    active.value = Set(new_json);
                    let _ = settings::Entity::update(active).exec(db).await;
                    tracing::info!("Migrated setting {} to new IP format ({} entries)", key, new_list.len());
                }
            }
        }
    }
}
