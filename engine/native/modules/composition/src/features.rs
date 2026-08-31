//! Feature 排序、去重、冲突检查（规格 §7.2）。
//!
//! 解析结果是对 catalog 的**完全划分**：请求的进 enabled，其余已知 feature 显式进
//! disabled。计划要自描述——留空白等于让下游猜默认值。

use std::collections::BTreeSet;

use crate::error::{err, invalid, CompositionError, CompositionErrorKind};
use crate::model::{FeatureCatalog, FeatureSet};

pub(crate) fn resolve(
    catalog: &FeatureCatalog,
    requested: &BTreeSet<String>,
) -> Result<FeatureSet, CompositionError> {
    let mut known = BTreeSet::new();
    for feature in &catalog.known {
        if feature.is_empty() {
            return Err(invalid("feature catalog 含空名".to_string()));
        }
        if !known.insert(feature.clone()) {
            return Err(invalid(format!("feature catalog 重复声明 {feature}")));
        }
    }

    for pair in &catalog.conflicts {
        for feature in pair {
            if !known.contains(feature) {
                return Err(invalid(format!("feature 互斥声明引用了未登记的 {feature}")));
            }
        }
        if pair[0] == pair[1] {
            return Err(invalid(format!("feature 互斥声明两侧相同：{}", pair[0])));
        }
    }

    for feature in requested {
        if !known.contains(feature) {
            return Err(err(
                CompositionErrorKind::UnknownFeature,
                format!("请求了未登记的 feature：{feature}"),
            ));
        }
    }

    for pair in &catalog.conflicts {
        if requested.contains(&pair[0]) && requested.contains(&pair[1]) {
            return Err(err(
                CompositionErrorKind::FeatureConflict,
                format!("feature 互斥：{} 与 {} 不能同时启用", pair[0], pair[1]),
            ));
        }
    }

    // BTreeSet 迭代即 UTF-8 字节序，排序与去重同时完成（ADR-0006 第 2 条）。
    let enabled: Vec<String> = requested.iter().cloned().collect();
    let disabled: Vec<String> = known
        .iter()
        .filter(|feature| !requested.contains(*feature))
        .cloned()
        .collect();

    Ok(FeatureSet { enabled, disabled })
}
