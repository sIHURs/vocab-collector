# 05: 导出 CSV 词汇清单

**What to build:** 用户选择保存位置，将全部未 Achieve、未删除的 Vocabulary Item 导出为可在电子表格中阅读的 CSV。

**Blocked by:** None (can start immediately)

**Status:** implemented-pending-acceptance

- [ ] 每行一个词，字段为词汇、源语言、当前显示译文、译文语言、学习状态、Encounter 次数、最近遇到时间；包含 Learning/Mastered/Paused。
- [ ] 不受当前搜索或分页限制，不包含上下文、Review 历史、设置、outbox 或撤销内部数据；明确这不是备份或无损恢复格式。
- [ ] 译文沿用当前显示语言规则；Starter Vocabulary Item 可导出，无 Encounter 时次数为 0、最近遇到时间为空。
- [ ] 提供专用批量投影，在一致读取范围内导出；可以复用已有优化，但不以 04 完成为强制依赖。
- [ ] 使用可靠 CSV 编码处理逗号、双引号、换行和 Unicode，选择并记录兼容 Excel 的 UTF-8 策略；对电子表格公式前缀做安全处理并说明其面向阅读的含义。
- [ ] 通过原生保存对话框选择文件，Rust 控制写入；只增加必要权限，不向前端开放任意文件系统操作。
- [ ] 安全处理已有文件、取消、权限失败、写入中断和暂存清理；失败不破坏原文件或留下误导性完整文件，只有成功才显示导出数量。
- [ ] 覆盖空词库、多语言、空译文、中文、多行、引号、公式前缀、范围筛选及失败路径的自动测试。

## Implementation record

主要实现已提交；验收清单保留供后续逐项核验，不代表全部完成。性能结果、测试记录和未验证边界见 [优化报告](../../../docs/database-p1-optimization-report.md)。Ticket 02 的压力规模并发保存等待仍需收尾；原生端到端及安装生命周期验证尚未执行。
