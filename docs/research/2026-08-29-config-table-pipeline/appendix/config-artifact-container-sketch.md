# Config Artifact Container Profile v1 — 非规范性草图

> 这是报告结论的实现提示，不是已冻结公共协议。正式落地前必须在架构仓形成独立规范、测试向量和安全审计。

## 逻辑层与物理层

- `SemanticRootHash`：对规范化逻辑值计算，独立于payload格式、压缩、chunk大小和文件打包。
- `ArtifactHash`：对具体manifest/chunk字节计算，用于下载完整性和内容寻址。
- `SourceRootHash`：对源与schema快照计算，用于审计，允许区分“显式默认”和“缺失后默认”。

## 建议目录

```text
ReleaseManifest
  releaseId
  schemaRootHash
  sourceRootHash
  serverProjectionRootHash
  clientProjectionRootHash
  voxelProjectionRootHash
  sharedPublicRootHash
  compilerProfile
  signatures[]

ProjectionManifest
  projectionId
  semanticRootHash
  requiredBootstrapChunks[]
  tableDescriptors[]

TableDescriptor
  tableId
  schemaVersion
  semanticTableHash
  layoutProfile
  keyProfile
  shardFunction
  primaryIndexChunkHash
  dataChunkHashes[]
  sidecarChunkHashes[]

ChunkDescriptor
  artifactHash
  codec
  compressedLength
  rawLength
  maxExpansionRatio
  alignment
  storageLocator
```

## 安全读取顺序

1. 检查Magic/Profile/SchemaVersion。
2. 检查所有长度、计数和乘法溢出。
3. 验签ReleaseManifest并检查版本/过期/回滚策略。
4. 对ProjectionManifest计算ArtifactHash和SemanticRoot承诺。
5. 请求chunk；验证compressed length与hash。
6. 在最大raw length/ratio限制内解压。
7. 验证payload结构、offset、alignment、UTF-8、enum。
8. 只在全部通过后发布为`VerifiedChunk`。

## Revision规则

- cache key包含`projection + revisionRoot + chunkHash`。
- 异步future捕获revision root；完成结果只能发布到同root命名空间。
- content hash完全相同的chunk可跨root共享物理buffer。
- Tick Barrier只交换Active root；旧root由epoch/refcount延迟释放。
- `TryGet`不发I/O；所有I/O进入`PrepareAsync/GetAsync`管理面。
