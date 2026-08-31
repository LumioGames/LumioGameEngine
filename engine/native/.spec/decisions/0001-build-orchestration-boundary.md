# 0001 · composition 只产不可变 BuildPlan，platform 是唯一构建执行入口

- 日期:2026-08-27
- 状态:生效

## 背景

架构 Review（CE-006）指出 composition 与 platform 同时声称拥有平台矩阵和构建产物:composition 输出「平台构建产物」，platform 也输出「平台 Native 包」。双重所有权导致 Compiler Flag、Feature、链接参数可能出现两个来源，Manifest 无法判断 Build Recipe 归属，同一 Target 可能产生不同目录布局和 Digest。

## 决策

- `composition` 只负责 Source Lock、Feature Resolution、构建参数与工具链版本冻结，产出**不可变 BuildPlan**（含 Input Digest）与 ProvenanceRecord;不执行编译，不拥有平台产物。
- `platform` 是**唯一权威构建执行入口**:规范化 TargetProfile，消费不可变 BuildPlan 执行编译/链接，产出 PlatformArtifactSet 与逐文件 ArtifactIndex;不得反向修改 BuildPlan。
- 不合并两个模块:「组合计划」与「平台执行」具有不同替换边界（Review 同一结论）。

## 后果

- 实际编译只有一个入口，CI 与本地共用;Build Recipe 归属唯一（composition）。
- BuildPlan 一经发布不可变，platform 需要的平台特化参数必须先回写 BuildPlan 生成流程，多一步流转成本。
- Digest Chain（Source → BuildPlan → ArtifactIndex）可形成单一来源链。
